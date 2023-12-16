/**
 * Example usage of the ThreadPool
 * Starg with the env variable "RUSTLOG=[level]" to get
 * logging info output.
 * fn main() {
 *     env_logger::init();
 *     let tp = ThreadPool::new(4);
 *     for i in 0..10 {
 *         debug!("Main thread: Sending {0}", i * i);
 *         tp.enqueue(move |id| {
 *             debug!("Worker {0}: Calculating Product of {1}", id, i);
 *             thread::sleep(Duration::from_millis(2000));
 *         });
 *     }
 * }
 *
 */
use log::{debug, error, info};
/// Thread Pool implementation
/// see main for example usage
///
use std::{
    sync::{
        mpsc::{channel, RecvError, Sender},
        Arc, Mutex,
    },
    thread,
};

type Job = Box<dyn FnOnce(usize) + Send + 'static>;

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

pub struct ThreadPool {
    send_queue: Option<Sender<Job>>,
    workers: Vec<Worker>,
}
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (tx, rx) = channel::<Job>();
        let rcv_channel = Arc::new(Mutex::new(rx));
        for i in 0..size {
            let rcv_queue = Arc::clone(&rcv_channel);
            let w = Worker {
                id: i + 1,
                thread: Some(thread::spawn(move || {
                    let id = i + 1;
                    info!("Worker {} started", id);
                    loop {
                        // make sure to avoid keeping the lock result in scope:
                        // the lock will be dropped once it goes out of scope,
                        // so we have to make sure this happens in this very next block,
                        // before we execute the job:
                        let mut msg: Option<Result<Job, RecvError>> = None;
                        {
                            match rcv_queue.lock() {
                                Ok(m) => {
                                    msg = Some(m.recv());
                                }
                                Err(e) => {
                                    error!("Worker {0}Error: {1}", id, e);
                                }
                            }
                        }
                        // Now, the lock should have expired, we execute the job
                        // if we get the lock and received the channel's job function:
                        if let Some(m) = msg {
                            match m {
                                Ok(worker_fn) => {
                                    debug!("Worker {0}: Start working ", id);
                                    worker_fn(id);
                                    debug!("Worker {0}: Finish working ", id);
                                }
                                Err(_) => {
                                    debug!("Thread {0} shutting down", id);
                                    break;
                                }
                            }
                        }
                    }
                })),
            };
            workers.push(w);
        }

        ThreadPool {
            workers,
            send_queue: Some(tx),
        }
    }

    pub fn enqueue<F: FnOnce(usize) + Send + 'static>(&self, task: F) {
        let job = Box::new(task);
        match &self.send_queue {
            Some(tx) => {
                tx.send(job).unwrap();
            }
            None => {}
        }
    }

    pub fn graceful_shutdown(&mut self) {
        drop(self.send_queue.take());
        for w in &mut self.workers {
            info!("Shutting down worker {}", w.id);
            let t = w.thread.take();
            if let Some(t) = t {
                t.join().unwrap();
            }
            // w.thread.take().unwrap().join().expect("Failed to join worker");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.graceful_shutdown();
    }
}
