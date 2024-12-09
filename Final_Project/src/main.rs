use std::collections::HashMap;
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

fn main() {
    let file_path = "Passenger_rail_data.csv"; // Replace with your file path

    // Step 1: Parse the CSV
    let data = parse_csv(file_path);

    // Step 2: Build node-level adjacency list
    let node_adjacency = build_node_adjacency(&data);

    // Step 3: Build county-level adjacency list
    let county_adjacency = build_county_adjacency(&data, &node_adjacency);

    // Step 4: Perform connectivity analysis on county-level adjacency list
    let county_connectivity = connectivity_analysis(&county_adjacency);

    // Print results
    println!("Top 20 most connected counties:");
    for (county, degree) in county_connectivity.iter().take(20) {
        println!("County: {}, Degree: {}", county, degree);
    }
}
