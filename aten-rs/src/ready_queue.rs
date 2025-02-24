// src/ready_queue.rs
use std::sync::{Arc, Mutex};
use std::collections::BinaryHeap;
use std::sync::Condvar;

// Import NodeTask from node_task.rs
use crate::node_task::NodeTask;

pub struct ReadyQueue {
    heap: Mutex<BinaryHeap<NodeTask>>,
    not_empty: Condvar,
}

impl ReadyQueue {
    pub fn new() -> Self {
        ReadyQueue {
            heap: Mutex::new(BinaryHeap::new()),
            not_empty: Condvar::new(),
        }
    }

    pub fn push(&self, item: NodeTask, increment_outstanding_tasks: bool) {
        let mut heap = self.heap.lock().unwrap();
        if increment_outstanding_tasks {
            // Logic to increment outstanding tasks (similar to C++)
            // Example: Some shared state, like a counter, that needs to be updated
        }
        heap.push(item);
        self.not_empty.notify_one();
    }

    pub fn push_shutdown_task(&self) {
        let mut heap = self.heap.lock().unwrap();
        heap.push(NodeTask::new(0)); // Create shutdown task
        self.not_empty.notify_one();
    }

    pub fn size(&self) -> usize {
        let heap = self.heap.lock().unwrap();
        heap.len()
    }

    pub fn pop(&self) -> NodeTask {
        let mut heap = self.heap.lock().unwrap();
        while heap.is_empty() {
            self.not_empty.wait(&mut heap).unwrap();
        }
        heap.pop().unwrap()
    }

    pub fn empty(&self) -> bool {
        let heap = self.heap.lock().unwrap();
        heap.is_empty()
    }
}

