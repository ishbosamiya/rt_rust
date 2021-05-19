use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Debug)]
struct Worker {
    _id: usize,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>) -> Self {
        let join_handle = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                WorkerMessage::RunNewJob(job) => {
                    job();
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
            .send(WorkerMessage::RunNewJob(job))
            .unwrap();
    }
}

/// The job to be executed by one of the threads
type Job = Box<dyn FnOnce() + Send + 'static>;

enum WorkerMessage {
    RunNewJob(Job),
    Terminate,
}

/// A Threadpool consists of a pool of threads to which Jobs
/// (functions) can be sent to execute on. The threads are 1:1 with
/// the operating system threads. A job can be sent to be executed and
/// a free thread picks up the job to complete.
#[derive(Debug)]
pub struct ThreadPool {
    num_threads: usize,
    workers: Vec<Worker>,
    sender: mpsc::Sender<WorkerMessage>,
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

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(num_threads);

        for i in 0..num_threads {
            workers.push(Worker::new(i, receiver.clone()));
        }

        return Self {
            num_threads,
            workers,
            sender,
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

        self.sender.send(WorkerMessage::RunNewJob(job)).unwrap();
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
    fn threadpool_scoped() {
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
}
