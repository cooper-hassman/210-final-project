use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufReader, BufRead};

/// Parse the CSV and return rows as a Vec<Vec<String>>
fn parse_csv(file_path: &str) -> Vec<Vec<String>> {
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);

    reader
        .lines()
        .skip(1) // Skip the header
        .filter_map(|line| line.ok()) // Ignore lines that can't be read
        .map(|line| line.split(',').map(|s| s.trim().to_string()).collect())
        .collect()
}

/// Build an adjacency list between nodes (FRFRANODE and TOFRANODE)
fn build_node_adjacency(data: &[Vec<String>]) -> HashMap<String, Vec<String>> {
    let mut node_adjacency: HashMap<String, Vec<String>> = HashMap::new();

    for row in data {
        let from_node = row[2].clone(); // FRFRANODE
        let to_node = row[3].clone(); // TOFRANODE

        node_adjacency.entry(from_node.clone()).or_default().push(to_node.clone());
        node_adjacency.entry(to_node).or_default().push(from_node);
    }

    node_adjacency
}

/// Build an adjacency list between counties (using STCNTYFIPS)
fn build_county_adjacency(
    data: &[Vec<String>],
    node_adjacency: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    let mut county_adjacency: HashMap<String, Vec<String>> = HashMap::new();

    // Map nodes to their counties
    let mut node_to_county: HashMap<String, String> = HashMap::new();
    for row in data {
        let node = row[2].clone(); // FRFRANODE
        let county = row[6].clone(); // STCNTYFIPS

        // Skip rows with empty county FIPS
        if !county.is_empty() {
            node_to_county.insert(node, county);
        }
    }

    // Build county-level adjacency based on node connections
    for (node, neighbors) in node_adjacency {
        if let Some(county) = node_to_county.get(node) {
            for neighbor in neighbors {
                if let Some(neighbor_county) = node_to_county.get(neighbor) {
                    if county != neighbor_county {
                        county_adjacency
                            .entry(county.clone())
                            .or_default()
                            .push(neighbor_county.clone());
                    }
                }
            }
        }
    }

    // Deduplicate adjacency lists
    for neighbors in county_adjacency.values_mut() {
        neighbors.sort();
        neighbors.dedup();
    }

    county_adjacency
}

/// Perform connectivity analysis on an adjacency list
fn connectivity_analysis(adjacency_list: &HashMap<String, Vec<String>>) -> Vec<(String, usize)> {
    let mut degrees: Vec<(String, usize)> = adjacency_list
        .iter()
        .map(|(node, neighbors)| (node.clone(), neighbors.len()))
        .collect();

    // Sort by degree in descending order
    degrees.sort_by(|a, b| b.1.cmp(&a.1));

    degrees
}

/// Perform BFS to calculate the size of a connected component
fn bfs_component_size(
    start: &String,
    adjacency_list: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
) -> usize {
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(start.clone());
    let mut size = 0;

    while let Some(node) = queue.pop_front() {
        if !visited.contains(&node) {
            visited.insert(node.clone());
            size += 1;
            if let Some(neighbors) = adjacency_list.get(&node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    size
}

fn largest_connected_component(adjacency_list: &HashMap<String, Vec<String>>) -> usize {
    let mut visited = HashSet::new();
    let mut max_size = 0;

    for node in adjacency_list.keys() {
        if !visited.contains(node) {
            let size = bfs_component_size(node, adjacency_list, &mut visited);
            if size > max_size {
                max_size = size;
            }
        }
    }

    max_size
}

/// Remove a county from the graph and return a new adjacency list
fn remove_county(
    county: &String,
    adjacency_list: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    let mut new_adjacency_list = adjacency_list.clone();
    new_adjacency_list.remove(county);

    // Remove the county from the adjacency lists of other counties
    for neighbors in new_adjacency_list.values_mut() {
        neighbors.retain(|neighbor| neighbor != county);
    }

    new_adjacency_list
}

/// Betweenness Centrality Analysis
fn betweenness_centrality(
    adjacency_list: &HashMap<String, Vec<String>>,
) -> Vec<(String, usize)> {
    let mut results = Vec::new();

    for county in adjacency_list.keys() {
        let modified_adjacency = remove_county(county, adjacency_list);
        let largest_component = largest_connected_component(&modified_adjacency);

        results.push((county.clone(), largest_component));
    }

    results.sort_by(|a, b| a.1.cmp(&b.1)); // Sort by impact (ascending)
    results
}

/// Compute the Average Shortest Path Length (ASPL) for the graph.
fn calculate_aspl(adjacency_list: &HashMap<String, Vec<String>>) -> f64 {
    let mut total_length = 0;
    let mut path_count = 0;

    for start in adjacency_list.keys() {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut distances = HashMap::new();

        queue.push_back(start.clone());
        visited.insert(start.clone());
        distances.insert(start.clone(), 0);

        while let Some(current) = queue.pop_front() {
            let current_distance = distances[&current];
            if let Some(neighbors) = adjacency_list.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push_back(neighbor.clone());
                        distances.insert(neighbor.clone(), current_distance + 1);
                    }
                }
            }
        }

        total_length += distances.values().sum::<usize>();
        path_count += distances.len() - 1; // Exclude the start node itself
    }

    total_length as f64 / path_count as f64
}

/// Remove a county from the graph and compute ASPL after removal.
/// Calculate ASPL after removing a specific county
fn calculate_aspl_with_removal(
    adjacency_list: &HashMap<String, Vec<String>>,
    county_to_remove: &str,
) -> f64 {
    // Remove the county from the adjacency list
    let modified_adjacency_list = remove_county(&county_to_remove.to_string(), adjacency_list);

    // Calculate ASPL on the modified adjacency list
    calculate_aspl(&modified_adjacency_list)
}

fn main() {
    let file_path = "Passenger_rail_data.csv";

    // Parse the CSV and get the adjacency list for counties
    let data = parse_csv(file_path);
    let node_adjacency = build_node_adjacency(&data);
    let county_adjacency = build_county_adjacency(&data, &node_adjacency);

    // Step 1: Top 20 Most Connected Counties by Degree using connectivity_analysis function
    println!("Top 20 Most Connected Counties by Degree:");
    let degrees = connectivity_analysis(&county_adjacency);
    for (county, degree) in degrees.iter().take(20) {
        println!("County: {}, Degree: {}", county, degree);
    }

    // Step 2: Top 20 Counties by Smallest Largest Connected Component Size (using largest_connected_component)
    println!("\nTop 20 Counties by Smallest Largest Connected Component Size:");
    let component_sizes = betweenness_centrality(&county_adjacency);
    for (county, component_size) in component_sizes.iter().take(20) {
        println!(
            "County: {}, Largest Component Size After Removal: {}",
            county, component_size
        );
    }

    // Step 3: Calculate ASPL for an Inputted County
    let county_to_analyze = "08001"; // Replace with your desired county FIPS code
    let aspl = calculate_aspl(&county_adjacency);
    println!("\nInitial ASPL: {:.3}", aspl);

    let aspl_for_county = calculate_aspl_with_removal(&county_adjacency, county_to_analyze);
    println!(
        "ASPL after removing county {}: {:.3}",
        county_to_analyze, aspl_for_county
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv() {
        let data = "FRFRANODE,TOFRANODE,STCNTYFIPS\nnode1,node2,10001\nnode2,node3,10002\n";
        let file_path = "test.csv";
        std::fs::write(file_path, data).expect("Unable to write test file");

        let rows = parse_csv(file_path);
        assert_eq!(rows.len(), 2); // Two rows after skipping header
        assert_eq!(rows[0][2], "10001");
        assert_eq!(rows[1][2], "10002");
    }

    #[test]
    fn test_build_node_adjacency() {
        let data = vec![
            vec!["node1".into(), "node2".into()],
            vec!["node2".into(), "node3".into()],
        ];

        let adjacency = build_node_adjacency(&data);
        assert!(adjacency.contains_key("node2"));
        assert_eq!(adjacency["node2"].len(), 2); // node2 connects to node1 and node3
    }

    #[test]
    fn test_build_county_adjacency() {
        let data = vec![
            vec!["".into(), "".into(), "node1".into(), "node2".into(), "".into(), "".into(), "10001".into()],
            vec!["".into(), "".into(), "node2".into(), "node3".into(), "".into(), "".into(), "10002".into()],
        ];
        let node_adjacency = build_node_adjacency(&data);
        let county_adjacency = build_county_adjacency(&data, &node_adjacency);

        assert!(county_adjacency.contains_key("10001"));
        assert_eq!(county_adjacency["10001"].len(), 1);
        assert!(county_adjacency["10001"].contains(&"10002".into()));
    }

    #[test]
    fn test_connectivity_analysis() {
        let mut adjacency = HashMap::new();
        adjacency.insert("10001".to_string(), vec!["10002".to_string()]);
        adjacency.insert("10002".to_string(), vec!["10001".to_string()]);

        let result = connectivity_analysis(&adjacency);
        assert_eq!(result[0].1, 2); // Component size is 2
    }

    #[test]
    fn test_bfs_component_size() {
        let mut adjacency = HashMap::new();
        adjacency.insert("A".to_string(), vec!["B".to_string()]);
        adjacency.insert("B".to_string(), vec!["A".to_string(), "C".to_string()]);
        adjacency.insert("C".to_string(), vec!["B".to_string()]);

        let mut visited = HashSet::new();
        let size = bfs_component_size(&"A".to_string(), &adjacency, &mut visited);
        assert_eq!(size, 3); // All nodes are connected
    }

    #[test]
    fn test_largest_connected_component() {
        let mut adjacency = HashMap::new();
        adjacency.insert("A".to_string(), vec!["B".to_string()]);
        adjacency.insert("B".to_string(), vec!["A".to_string(), "C".to_string()]);
        adjacency.insert("C".to_string(), vec!["B".to_string()]);
        adjacency.insert("D".to_string(), vec!["E".to_string()]);
        adjacency.insert("E".to_string(), vec!["D".to_string()]);

        let size = largest_connected_component(&adjacency);
        assert_eq!(size, 3); // Largest component is A-B-C
    }

    #[test]
    fn test_remove_county() {
        let mut adjacency = HashMap::new();
        adjacency.insert("10001".to_string(), vec!["10002".to_string(), "10003".to_string()]);
        adjacency.insert("10002".to_string(), vec!["10001".to_string()]);
        adjacency.insert("10003".to_string(), vec!["10001".to_string()]);

        let modified = remove_county(&"10001".to_string(), &adjacency);
        assert!(!modified.contains_key("10001"));
        assert_eq!(modified["10002"].len(), 0);
        assert_eq!(modified["10003"].len(), 0);
    }

    #[test]
    fn test_calculate_aspl() {
        let mut adjacency = HashMap::new();
        adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        adjacency.insert("B".to_string(), vec!["A".to_string()]);
        adjacency.insert("C".to_string(), vec!["A".to_string()]);

        let aspl = calculate_aspl(&adjacency);
        assert!((aspl - 1.333).abs() < 0.001); // ASPL should match expected value
    }

    #[test]
    fn test_calculate_aspl_with_removal() {
        let mut adjacency = HashMap::new();
        adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        adjacency.insert("B".to_string(), vec!["A".to_string()]);
        adjacency.insert("C".to_string(), vec!["A".to_string()]);

        let aspl = calculate_aspl_with_removal(&adjacency, "A");
        assert!(aspl.is_nan() || aspl == 0.0); // Removing A disconnects everything
    }
}
