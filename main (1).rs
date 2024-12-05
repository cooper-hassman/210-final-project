// outline
struct Graph {
    adjacency_list: HashMap<String, Vec<String>>, // Node -> Connected Nodes
}

impl Graph {
    fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: String, to: String) {
        self.adjacency_list.entry(from.clone()).or_insert(vec![]).push(to.clone());
        self.adjacency_list.entry(to).or_insert(vec![]).push(from); // For undirected graph
    }
}

// BFS outline
fn bfs(graph: &Graph, start: &str, end: &str) -> Option<usize> {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    queue.push_back((start, 0)); // Node, Depth
    visited.insert(start.to_string());

    while let Some((current, depth)) = queue.pop_front() {
        if current == end {
            return Some(depth);
        }

        if let Some(neighbors) = graph.adjacency_list.get(current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.to_string());
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }
    }
    None
}


fn main() {
    println!("Hello, world!");
}
