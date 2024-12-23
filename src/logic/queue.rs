//! The queue module contains the job queue implementation for the script
//! engine. This allows for async execution of JavaScript promises and futures.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use boa_engine::Context;
use boa_engine::job::{FutureJob, JobQueue, NativeJob};
use futures_util::stream::FuturesUnordered;
use smol::stream::StreamExt;
use smol::{LocalExecutor, future};

/// A listener that signals the script engine to shut down. This is simply a
/// boolean value that is shared between the main thread and the script engine
/// thread.
///
/// This value is shared between the main thread and the script engine thread.
/// Cloning this value creates a new reference to the same shutdown listener.
///
/// Note that triggering a shutdown does not immediately stop the script engine.
/// Any running jobs will be allowed to finish before the engine stops, but all
/// new jobs, or queued jobs, will be ignored.
#[derive(Debug, Default, Clone)]
pub struct ScriptEngineShutdown(Arc<Mutex<bool>>);

impl ScriptEngineShutdown {
    /// Creates a new shutdown listener.
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(false)))
    }

    /// Shuts down the script engine.
    pub fn shutdown(&self) {
        *self.0.lock().unwrap() = true;
    }

    /// Returns whether the script engine has been shut down.
    pub fn is_shutdown(&self) -> bool {
        *self.0.lock().unwrap()
    }
}

/// The queue struct is responsible for managing the execution of jobs.
pub struct ScriptEngineJobQueue {
    /// The futures that are currently running.
    futures: RefCell<FuturesUnordered<FutureJob>>,

    /// The jobs that are currently queued.
    jobs: RefCell<VecDeque<NativeJob>>,

    /// The shutdown listener signals the script engine.
    shutdown: ScriptEngineShutdown,
}

impl ScriptEngineJobQueue {
    /// Creates a new queue with the given executor and communication channels.
    pub fn new(shutdown: ScriptEngineShutdown) -> Self {
        Self {
            futures: Default::default(),
            jobs: Default::default(),
            shutdown,
        }
    }
}

impl JobQueue for ScriptEngineJobQueue {
    fn enqueue_promise_job(&self, job: NativeJob, _: &mut Context) {
        self.jobs.borrow_mut().push_back(job);
    }

    fn enqueue_future_job(&self, future: FutureJob, _: &mut Context) {
        self.futures.borrow().push(future);
    }

    fn run_jobs(&self, context: &mut Context) {
        let context = RefCell::new(context);

        let executor = LocalExecutor::new();
        future::block_on(executor.run(async {
            let fut_queue = async {
                loop {
                    if self.shutdown.is_shutdown() {
                        return;
                    }

                    if self.futures.borrow().is_empty() {
                        future::yield_now().await;
                        continue;
                    }

                    let futures = &mut std::mem::take(&mut *self.futures.borrow_mut());
                    while let Some(job) = futures.next().await {
                        self.enqueue_promise_job(job, &mut context.borrow_mut());
                    }
                }
            };

            let job_queue = async {
                loop {
                    if self.shutdown.is_shutdown() {
                        return;
                    }

                    if self.jobs.borrow().is_empty() {
                        future::yield_now().await;
                        continue;
                    };

                    let jobs = std::mem::take(&mut *self.jobs.borrow_mut());
                    for job in jobs {
                        if let Err(e) = job.call(&mut context.borrow_mut()) {
                            eprintln!("Uncaught {e}");
                        }
                        future::yield_now().await;
                    }
                }
            };

            future::zip(fut_queue, job_queue).await;
        }));
    }
}
