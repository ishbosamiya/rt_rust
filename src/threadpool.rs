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

pub struct ThreadPool {
    num_threads: usize,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        assert_ne!(num_threads, 0);

        let mut workers = Vec::with_capacity(num_threads);

        for id in 0..num_threads {
            workers.push(Worker::new(id));
        }

        return Self { num_threads };
    }

    pub fn get_num_threads(&self) -> usize {
        return self.num_threads;
    }
}
