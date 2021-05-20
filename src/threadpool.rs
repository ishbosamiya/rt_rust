use crossbeam_channel::{bounded, Receiver, Sender};

use std::thread;

#[derive(Debug)]
struct Worker {
    _id: usize,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Receiver<WorkerMessage>,
        scope_receiver: Receiver<ScopeMessage>,
        scope_sender: Sender<ScopeMessage>,
    ) -> Self {
        let join_handle = thread::spawn(move || loop {
            // println!("before message id: {}", id);
            let message = receiver.recv().unwrap();
            // println!("after message id: {}, message: {:?}", id, message);

            match message {
                WorkerMessage::RunNewJob(job) => {
                    job.0();
                }
                WorkerMessage::TerminateScoped => {
                    scope_sender.send(ScopeMessage::ReceivedTerminate).unwrap();
                    // println!("id: {} waiting for scope message", id);
                    let scope_message = scope_receiver.recv().unwrap();
                    // println!("id: {} got scope message: {:?}", id, scope_message);
                    match scope_message {
                        ScopeMessage::AllTerminated => {}
                        _ => {}
                    }
                }
                WorkerMessage::Terminate => {
                    break;
                }
            }
        });
        return Self {
            _id: id,
            join_handle: Some(join_handle),
        };
    }
}

#[derive(Debug, Clone)]
pub struct Scope<'a> {
    pool: &'a ThreadPool,
}

impl<'a> Scope<'a> {
    fn new(pool: &'a ThreadPool) -> Self {
        Self { pool }
    }

    pub fn execute<'scope, F>(&self, f: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        let job;
        unsafe {
            job = std::mem::transmute::<
                Box<dyn FnOnce() + Send + 'scope>,
                Box<dyn FnOnce() + Send + 'static>,
            >(Box::new(f));
        }

        self.pool
            .sender
            .send(WorkerMessage::RunNewJob(Job(job)))
            .unwrap();
    }
}

/// The job to be executed by one of the threads
struct Job(Box<dyn FnOnce() + Send + 'static>);

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("job")
    }
}

#[derive(Debug)]
enum WorkerMessage {
    RunNewJob(Job),
    TerminateScoped,
    Terminate,
}

#[derive(Debug)]
enum ScopeMessage {
    ReceivedTerminate,
    AllTerminated,
}

/// A Threadpool consists of a pool of threads to which Jobs
/// (functions) can be sent to execute on. The threads are 1:1 with
/// the operating system threads. A job can be sent to be executed and
/// a free thread picks up the job to complete.
#[derive(Debug)]
pub struct ThreadPool {
    num_threads: usize,
    workers: Vec<Worker>,
    sender: Sender<WorkerMessage>,
    scope_sender: Sender<ScopeMessage>,
    scope_receiver: Receiver<ScopeMessage>,
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// size: number of threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero
    pub fn new(num_threads: usize) -> Self {
        assert_ne!(num_threads, 0);

        let (sender, receiver) = bounded(num_threads * 2);
        let (scope_sender, scope_receiver) = bounded(num_threads * 2);
        let (scope_sender_2, scope_receiver_2) = bounded(num_threads * 2);

        let mut workers = Vec::with_capacity(num_threads);

        for i in 0..num_threads {
            workers.push(Worker::new(
                i,
                receiver.clone(),
                scope_receiver.clone(),
                scope_sender_2.clone(),
            ));
        }

        return Self {
            num_threads,
            workers,
            sender,
            scope_sender,
            scope_receiver: scope_receiver_2,
        };
    }

    /// Execute the given job `f`.
    ///
    /// # Panics
    ///
    /// Panics if the internal communication pipeline closes which may
    /// happen if one of the currently running jobs panics
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .send(WorkerMessage::RunNewJob(Job(job)))
            .unwrap();
    }

    /// Create new scoped threadpool with `num_threads` number of threads
    ///
    /// Use Scope to execute the threads
    ///
    /// When `scoped` is dropped, the main thread will be blocked
    /// until all the spawned threads join back
    pub fn new_scoped<'scope, F>(num_threads: usize, f: F)
    where
        F: FnOnce(Scope) + Send + 'scope,
    {
        let pool = Self::new(num_threads);
        let scope = Scope::new(&pool);
        f(scope);
    }

    /// Give a scoped region to be able to spawn functions
    ///
    /// Before exiting `scoped`, all functions ever run need to
    /// complete, so better to create a threadpool utilized only for
    /// scoped function execution
    pub fn scoped<'scope, F>(&self, f: F)
    where
        F: FnOnce(Scope) + Send + 'scope,
    {
        let scope = Scope::new(self);
        f(scope);

        self.workers
            .iter()
            .for_each(|_| self.sender.send(WorkerMessage::TerminateScoped).unwrap());

        let _all_termintated: Vec<_> = self
            .scope_receiver
            .iter()
            .take(self.workers.len())
            .collect();

        self.workers
            .iter()
            .for_each(|_| self.scope_sender.send(ScopeMessage::AllTerminated).unwrap());
    }

    pub fn get_num_threads(&self) -> usize {
        return self.num_threads;
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.workers
            .iter()
            .for_each(|_| self.sender.send(WorkerMessage::Terminate).unwrap());

        self.workers.iter_mut().for_each(|worker| {
            if let Some(join_handle) = worker.join_handle.take() {
                join_handle.join().unwrap();
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn threadpool_test() {
        let num_threads = 5;
        let pool = ThreadPool::new(num_threads);

        let (tx, rx) = mpsc::channel();

        for _ in 0..pool.get_num_threads() {
            let tx = tx.clone();
            pool.execute(move || {
                tx.send(1).unwrap();
            });
        }

        assert_eq!(
            rx.iter().take(pool.get_num_threads()).fold(0, |a, b| a + b),
            pool.get_num_threads()
        );
    }

    #[test]
    fn threadpool_test_2() {
        let num_threads = 200;
        let (tx, rx) = mpsc::channel();
        let num = 500;

        {
            let pool = ThreadPool::new(num_threads);
            for _ in 0..num {
                let tx = tx.clone();
                pool.execute(move || {
                    tx.send(1).unwrap();
                });
            }
        }

        assert_eq!(rx.iter().take(num).fold(0, |a, b| a + b), num);
    }

    #[test]
    fn threadpool_new_scoped() {
        let num_threads = 5;

        let v = vec![1, 2, 3, 4, 5, 6];
        let v_ref = &v;
        let num_jobs = v.len();
        let (tx, rx) = mpsc::channel();
        ThreadPool::new_scoped(num_threads, move |scope| {
            for i in 0..num_jobs {
                let tx = tx.clone();
                scope.execute(move || {
                    tx.send(v_ref[i]).unwrap();
                });
            }
        });

        assert_eq!(
            rx.iter().take(num_jobs).fold(0, |a, b| a + b),
            v.iter().fold(0, |a, b| a + b)
        );
    }

    #[test]
    fn threadpool_scoped() {
        let num_threads = 5;
        let pool = ThreadPool::new(num_threads);

        let v = vec![1, 2, 3, 4, 5, 6];
        let v_ref = &v;
        let num_jobs = v.len();
        let (tx, rx) = mpsc::channel();
        pool.scoped(move |scope| {
            for i in 0..num_jobs {
                let tx = tx.clone();
                scope.execute(move || {
                    tx.send(v_ref[i]).unwrap();
                });
            }
        });

        assert_eq!(
            rx.iter().take(num_jobs).fold(0, |a, b| a + b),
            v.iter().fold(0, |a, b| a + b)
        );
    }
}
