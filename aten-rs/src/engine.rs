use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

pub struct Engine;

impl Engine {
    pub fn thread_main(graph_task: Option<Arc<GraphTask>>) {
        while let Some(task) = graph_task.as_ref() {
            if task.future_result.completed() {
                break;
            }

            if let Some(local_task) = local_ready_queue.pop().and_then(|t| t.base.upgrade()) {
                if local_task.is_shutdown_task {
                    log::info!("Engine thread shutting down");
                    break;
                }
                
                set_device(worker_device);
                if let Some(ref func) = local_task.fn_ {
                    if !local_task.has_error.load(Ordering::Relaxed) {
                        let _tls_guard = ThreadLocalStateGuard::new(&local_task.thread_locals);
                        let _warnings_guard = WarningHandlerGuard::new(&local_task.warning_handler);

                        if let Err(e) = Self::evaluate_function(&local_task, func, &local_task.inputs) {
                            Self::thread_on_exception(&local_task, func, e);
                        }
                    }
                }

                local_task.decrement_outstanding_tasks();

                if local_task.completed() {
                    local_task.mark_as_completed_and_run_post_processing();
                    if worker_device != local_task.owner {
                        ready_queue_by_index(local_task.cpu_ready_queue, local_task.owner)
                            .push(NodeTask::new(local_task, None, InputBuffer::new(0)));
                    }
                }
            }
        }
    }

    pub fn reentrant_thread_init() {
        set_terminate_handler();
        init_num_threads();

        loop {
            let mut thread_pool_shared = thread_pool_shared.lock().unwrap();
            thread_pool_shared.num_workers += 1;
            
            while thread_pool_shared.graphtasks_queue.is_empty() {
                thread_pool_shared.work.wait(&mut thread_pool_shared);
            }
            
            thread_pool_shared.num_workers -= 1;
            if let Some(task) = thread_pool_shared.graphtasks_queue.pop_front().and_then(|t| t.upgrade()) {
                set_device(task.owner);
                local_ready_queue = ready_queue_by_index(task.cpu_ready_queue, task.owner);
                Self::thread_main(Some(task));
            } else {
                log::info!("GraphTask expired, skipping execution");
            }
        }
    }

    pub fn thread_on_exception(graph_task: &Arc<GraphTask>, fn_: &Node, e: Box<dyn std::error::Error>) {
        graph_task.set_exception(e, fn_);
    }

    pub fn execute_with_graph_task(
        &self,
        graph_task: Arc<GraphTask>,
        graph_root: Arc<Node>,
        input_buffer: InputBuffer,
    ) -> Arc<Future> {
        self.initialize_device_threads_pool();
        
        {
            let mut lock = graph_task.mutex.lock().unwrap();
            let queue = ready_queue(graph_task.cpu_ready_queue.clone(), graph_root.device());
            
            if self.worker_device == NO_DEVICE {
                self.set_device(CPU_DEVICE);
                graph_task.owner = self.worker_device;
                queue.push(NodeTask::new(graph_task.clone(), graph_root.clone(), input_buffer));
                drop(lock);
                Self::thread_main(Some(graph_task.clone()));
                assert!(graph_task.future_result.lock().unwrap().completed());
                self.worker_device = NO_DEVICE;
            } else {
                graph_task.owner = self.worker_device;
                queue.push(NodeTask::new(graph_task.clone(), graph_root.clone(), input_buffer));
                
                if self.current_depth >= self.max_recursion_depth {
                    add_thread_pool_task(graph_task.clone());
                } else {
                    self.total_depth.fetch_add(1, Ordering::SeqCst);
                    self.current_depth += 1;
                    drop(lock);
                    Self::thread_main(Some(graph_task.clone()));
                    self.current_depth -= 1;
                    self.total_depth.fetch_sub(1, Ordering::SeqCst);
                    assert!(graph_task.future_result.lock().unwrap().completed());
                }
            }
        }
        graph_task.future_result.clone()
    }
}
