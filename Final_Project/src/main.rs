mod data_cleaning {
    use std::fs::File;
    use std::io::{BufReader, BufRead};

    pub fn parse_csv(file_path: &str) -> Vec<Vec<String>> {
        let file = File::open(file_path).expect("Can't open file");
        let reader = BufReader::new(file);

        reader.lines().skip(1) // skips the column titles
            .filter_map(|line| line.ok()) // gets rid of unreadable lines
            .map(|line| line.split(',').map(|s| s.trim().to_string()).collect()) // splits by commas and collects
            .collect()
    }
}

mod adjacency_lists {
    use std::collections::HashMap;

    pub fn build_node_adjacency(data: &[Vec<String>]) -> HashMap<String, Vec<String>> {
        let mut node_adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for row in data {
            let from_node = row[2].clone(); // FRFRANODE - from node
            let to_node = row[3].clone(); // TOFRANODE - to node

            node_adjacency.entry(from_node.clone()).or_default().push(to_node.clone());
            node_adjacency.entry(to_node).or_default().push(from_node);
        }
        node_adjacency
    }

    pub fn build_county_adjacency(data: &[Vec<String>], node_adjacency: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, Vec<String>> {
        let mut county_adjacency: HashMap<String, Vec<String>> = HashMap::new();
        let mut node_to_county: HashMap<String, String> = HashMap::new();
        
        //initialized HashMaps for county and node_to_county adjacency lists
        for row in data {
            let node = row[2].clone(); // FRFRANODE - from node
            let county = row[6].clone(); // STCNTYFIPS - state and county ID
            if !county.is_empty() { // skips over rows where state/county ID is missing
                node_to_county.insert(node, county);
            }
        }

        for (node, neighbors) in node_adjacency { // iterating through node_adjacency HashMap
            if let Some(county) = node_to_county.get(node) { // node's county
                for neighbor in neighbors {
                    if let Some(neighbor_county) = node_to_county.get(neighbor) { // neighboring nodes' counties
                        if county != neighbor_county {
                            county_adjacency.entry(county.clone()).or_default().push(neighbor_county.clone());
                            // pushes connected counties to county_adjacency HashMap
                        }
                    }
                }
            }
        }

        for neighbors in county_adjacency.values_mut() {
            neighbors.sort();
            neighbors.dedup(); 
        } // deletes repeated values in each county's adjacent counties vector

        county_adjacency
    }

    pub fn remove_node(node: &String, adjacency_list: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, Vec<String>> {
        let mut new_adjacency_list = adjacency_list.clone();
        new_adjacency_list.remove(node); // removes node from keys
        for neighbors in new_adjacency_list.values_mut() {
            neighbors.retain(|neighbor| neighbor != node);
        } // removes node from each value
        new_adjacency_list
    }
}

mod graph_analysis {
    use std::collections::{HashMap, HashSet, VecDeque};
    use crate::adjacency_lists::remove_node;

    pub fn connectivity_analysis(adjacency_list: &HashMap<String, Vec<String>>) -> Vec<(String, usize)> {
        let mut num_neighbors: Vec<(String, usize)> = adjacency_list
            .iter()
            .map(|(node, neighbors)| (node.clone(), neighbors.len()))
            .collect();
            // finds length of each neighbor vector
        // sort by number of neighbors in descending order
        num_neighbors.sort_by(|a, b| b.1.cmp(&a.1));
        num_neighbors // returns how many nodes each node is connected to
    }

    fn bfs_component_size(
        start: &String,
        adjacency_list: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
    ) -> usize {
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(start.clone());
        let mut size = 0;
        // initializes queue and size variable
        while let Some(node) = queue.pop_front() {
            if !visited.contains(&node) { 
                visited.insert(node.clone());
                // if node is not visited yet, add it to visited HashSet
                size += 1; // increment size variable
                if let Some(neighbors) = adjacency_list.get(&node) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            queue.push_back(neighbor.clone());
                        } // adds all unvisited neighbors of that node to queue
                    }
                }
            }
        } // loop ends when queue is empty; all nodes within component visited
        size
    }

    pub fn largest_connected_component(adjacency_list: &HashMap<String, Vec<String>>) -> usize {
        let mut visited = HashSet::new();
        let mut max_size = 0;
        // initialize visited HashSet and max_size variable
        for node in adjacency_list.keys() {
            if !visited.contains(node) {
                let size = bfs_component_size(node, adjacency_list, &mut visited);
                // find component size of each unvisited node
                if size > max_size {
                    max_size = size;
                } // replace max_size with size of connected component if greater
            }
        }
        max_size
    }

    pub fn betweenness_centrality(
        adjacency_list: &HashMap<String, Vec<String>>,
    ) -> Vec<(String, usize)> {
        let mut results = Vec::new();
        // initialize results vector
        for node in adjacency_list.keys() {
            let modified_adjacency = remove_node(node, adjacency_list);
            let largest_component = largest_connected_component(&modified_adjacency);
            // creates new adjacency list without node, then calculates largest conn. comp. of new graph
            results.push((node.clone(), largest_component));
        }
        results.sort_by(|a, b| a.1.cmp(&b.1)); 
        // sorts results in ascending order to see which nodes' removal affected connectivity the most
        results
    }
}

fn main() {
    let file_path = "Passenger_rail_data.csv";
    let data = data_cleaning::parse_csv(file_path);
    let node_adjacency = adjacency_lists::build_node_adjacency(&data);
    let county_adjacency = adjacency_lists::build_county_adjacency(&data, &node_adjacency);
    // parse csv and load adjacency lists for nodes and counties
    
    println!("Top 20 Most Connected Counties by Number of Neighbors:");
    let num_neighbors = graph_analysis::connectivity_analysis(&county_adjacency);
    for (county, neighbors) in num_neighbors.iter().take(20) {
        println!("County: {}, Number of Neighbors: {}", county, neighbors);
    } // uses connectivity_analysis to find and print most connected counties

    let initial_largest = graph_analysis::largest_connected_component(&county_adjacency);
    println!("\nLargest Connected Component without Removal: {}", initial_largest);
    println!("\nTop 20 Counties by Smallest Largest Connected Component Size after Removal:");
    let component_sizes = graph_analysis::betweenness_centrality(&county_adjacency);
    for (county, component_size) in component_sizes.iter().take(20) {
        println!(
            "County: {}, Largest Component Size After Removal: {}",
            county, component_size
        );
    } // uses betweenness_centrality to find and print counties with smallest largest connected components after removal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_cleaning;
    use crate::adjacency_lists;
    use crate::graph_analysis;
    use std::collections::HashMap;
    use std::collections::HashSet;

    #[test]
    fn test_parse_csv() {
        let data = "FRFRANODE,TOFRANODE,STCNTYFIPS\nnode1,node2,10001\nnode2,node3,10002\n";
        let file_path = "test.csv";
        std::fs::write(file_path, data).expect("Unable to write test file");

        let rows = data_cleaning::parse_csv(file_path);
        assert_eq!(rows.len(), 2); // two rows after skipping column titles
        assert_eq!(rows[0][2], "10001");
        assert_eq!(rows[1][2], "10002");
    }

    #[test]
    fn test_build_node_adjacency() {
        let data = vec![
            vec!["".into(), "".into(), "node1".into(), "node2".into()],
            vec!["".into(), "".into(), "node2".into(), "node3".into()],
        ];

        let adjacency = adjacency_lists::build_node_adjacency(&data);
        assert!(adjacency.contains_key("node2"));
        assert_eq!(adjacency["node2"].len(), 2); 
        // node2 connects to node1 and node3
    }

    #[test]
    fn test_build_county_adjacency() {
        let data = vec![
            vec!["".into(), "".into(), "node1".into(), "node2".into(), "".into(), "".into(), "10001".into()],
            vec!["".into(), "".into(), "node2".into(), "node3".into(), "".into(), "".into(), "10002".into()],
        ];
        let node_adjacency = adjacency_lists::build_node_adjacency(&data);
        let county_adjacency = adjacency_lists::build_county_adjacency(&data, &node_adjacency);

        assert!(county_adjacency.contains_key("10001"));
        assert_eq!(county_adjacency["10001"].len(), 1);
        assert!(county_adjacency["10001"].contains(&"10002".into()));
        // makes sure county connections are computed correctly
    }

    #[test]
    fn test_connectivity_analysis() {
        let mut adjacency = HashMap::new();
        adjacency.insert("10001".to_string(), vec!["10002".to_string()]);
        adjacency.insert("10002".to_string(), vec!["10001".to_string()]);

        let result = graph_analysis::connectivity_analysis(&adjacency);
        assert_eq!(result[0].1, 1); // node 10001 has one connection
    }

    #[test]
    fn test_largest_connected_component() {
        let mut adjacency = HashMap::new();
        adjacency.insert("A".to_string(), vec!["B".to_string()]);
        adjacency.insert("B".to_string(), vec!["A".to_string(), "C".to_string()]);
        adjacency.insert("C".to_string(), vec!["B".to_string()]);
        adjacency.insert("D".to_string(), vec!["E".to_string()]);
        adjacency.insert("E".to_string(), vec!["D".to_string()]);

        let size = graph_analysis::largest_connected_component(&adjacency);
        assert_eq!(size, 3); // largest component is A-B-C
    }

    #[test]
    fn test_remove_node() {
        let mut adjacency = HashMap::new();
        adjacency.insert("10001".to_string(), vec!["10002".to_string(), "10003".to_string()]);
        adjacency.insert("10002".to_string(), vec!["10001".to_string()]);
        adjacency.insert("10003".to_string(), vec!["10001".to_string()]);

        let modified = adjacency_lists::remove_node(&"10001".to_string(), &adjacency);
        assert!(!modified.contains_key("10001"));
        assert_eq!(modified["10002"].len(), 0);
        assert_eq!(modified["10003"].len(), 0); 
        // removes node from keys and values of adjacency list
    }

    #[test]
    fn test_betweenness_centrality() {
        let mut adjacency = HashMap::new();
        adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        adjacency.insert("B".to_string(), vec!["A".to_string(), "C".to_string()]);
        adjacency.insert("C".to_string(), vec!["A".to_string(), "B".to_string()]);

        let centrality = graph_analysis::betweenness_centrality(&adjacency);
        let expected = vec![
            ("A".to_string(), 2), // remove A, B-C connected
            ("B".to_string(), 2), // remove B, A-C connected
            ("C".to_string(), 2), // remove C, A-B connected
        ];
        assert_eq!(centrality, expected);
    }
}