// src/node_task.rs
#[derive(Debug, Clone)]
pub struct NodeTask {
    // Define the fields for NodeTask
    pub id: u32, // Example field, adjust as needed
    // Other fields as needed
}

impl NodeTask {
    pub fn new(id: u32) -> Self {
        NodeTask { id }
    }
}

