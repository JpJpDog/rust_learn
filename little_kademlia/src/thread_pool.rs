use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce(usize) + Send + 'static>;

struct Status {
    queue: VecDeque<Job>,
    shutdown: bool,
}

type Notifier = Arc<(Mutex<Status>, Condvar)>;

fn next_job(notifier: &Notifier, id: usize) -> Option<Job> {
    let (lock, cvar) = &**notifier;
    let mut status = lock.lock().unwrap();
    loop {
        match status.queue.pop_front() {
            None => {
                if status.shutdown {
                    return None;
                }
                println!("worker {} is waiting a job", id);
                status = cvar.wait(status).unwrap();
            }
            some => return some,
        }
    }
}

struct Worker {
    idx: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, notifier: Notifier) -> Self {
        Self {
            idx: id,
            thread: Some(thread::spawn(move || loop {
                if let Some(job) = next_job(&notifier, id) {
                    println!("Worker {} get a job. executing...", id);
                    job(id);
                } else {
                    break;
                }
            })),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    notifier: Notifier,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let status = Status {
            queue: VecDeque::new(),
            shutdown: false,
        };
        let notifier = Arc::new((Mutex::new(status), Condvar::new()));
        let mut workers = Vec::new();
        for i in 0..size {
            workers.push(Worker::new(i, notifier.clone()));
        }
        Self { workers, notifier }
    }

    pub fn execute<F: FnOnce(usize) + Send + 'static>(&self, f: F) {
        let (lock, cvar) = &*self.notifier;
        let mut status = lock.lock().unwrap();
        status.queue.push_back(Box::new(f));
        cvar.notify_one();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.notifier;
        let mut status = lock.lock().unwrap();
        status.shutdown = true;
        println!("Sending terminate msg to all workers.");
        cvar.notify_all();
        std::mem::drop(status);
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.idx);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(8);
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
        pool.execute(|_idx| println!("hello"));
    }
}
