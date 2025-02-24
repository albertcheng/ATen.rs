use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

pub struct Engine;

impl Engine {
    pub fn thread_main(graph_task: Option<Arc<GraphTask>>) {
        while graph_task.as_ref().map_or(true, |task| !task.future_result.completed()) {
            let local_graph_task;
            {
                let task = local_ready_queue.pop();
                if task.is_shutdown_task {
                    log::info!("Engine thread shutting down");
                    break;
                }
                
                local_graph_task = task.base.upgrade();
                if local_graph_task.is_none() {
                    continue;
                }

                set_device(worker_device);

                let local_graph_task = local_graph_task.unwrap();
                if let Some(ref fn_) = task.fn_ {
                    if !local_graph_task.has_error.load(Ordering::Relaxed) {
                        let _tls_guard = ThreadLocalStateGuard::new(&local_graph_task.thread_locals);
                        let _warnings_guard = WarningHandlerGuard::new(&local_graph_task.warning_handler);
                        
                        if let Err(e) = Self::evaluate_function(&local_graph_task, fn_, &task.inputs) {
                            Self::thread_on_exception(&local_graph_task, fn_, e);
                        }
                    }
                }
            }
            
            local_graph_task.decrement_outstanding_tasks();
            
            if local_graph_task.completed() {
                local_graph_task.mark_as_completed_and_run_post_processing();
                if worker_device != local_graph_task.owner {
                    ready_queue_by_index(local_graph_task.cpu_ready_queue, local_graph_task.owner)
                        .push(NodeTask::new(local_graph_task, None, InputBuffer::new(0)));
                }
            }
        }
    }

    pub fn reentrant_thread_init() {
        set_terminate_handler();
        init_num_threads();
        loop {
            let mut tp_shared = thread_pool_shared.lock().unwrap();
            tp_shared.num_workers += 1;
            tp_shared.work.wait(|tp_shared| !tp_shared.graphtasks_queue.is_empty());
            tp_shared.num_workers -= 1;
            
            let task = tp_shared.graphtasks_queue.pop_front();
            drop(tp_shared);
            
            if let Some(task) = task {
                let graph_task = task.upgrade();
                if graph_task.is_none() {
                    log::info!("GraphTask expired, skipping execution");
                    continue;
                }
                let graph_task = graph_task.unwrap();
                set_device(graph_task.owner);
                local_ready_queue = ready_queue_by_index(graph_task.cpu_ready_queue, graph_task.owner);
                thread_main(Some(graph_task));
            }
        }
    }

    pub fn thread_on_exception(graph_task: &Arc<GraphTask>, fn_: &Node, e: std::error::Error) {
        graph_task.set_exception(Box::new(e), fn_);
    }
}

