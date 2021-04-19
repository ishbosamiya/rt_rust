use std::sync::mpsc;
use std::thread;

struct Worker {
    id: usize,
    join_handle: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize) -> Self {
        return Self {
            id,
            join_handle: thread::spawn(|| {}),
        };
    }
}

/// The job to be executed by one of the threads
type Job = Box<dyn FnOnce() + Send + 'static>;

/// A Threadpool consists of a pool of threads to which Jobs
/// (functions) can be sent to execute on. The threads are 1:1 with
/// the operating system threads. A job can be sent to be executed and
/// a free thread picks up the job to complete.
pub struct ThreadPool {
    num_threads: usize,
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
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

        let mut workers = Vec::with_capacity(num_threads);

        for id in 0..num_threads {
            workers.push(Worker::new(id));
        }

        return Self {
            num_threads,
            workers,
            sender,
        };
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // TODO(ish)
    }

    pub fn get_num_threads(&self) -> usize {
        return self.num_threads;
    }
}
