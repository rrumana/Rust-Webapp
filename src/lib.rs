use log::{error, debug};
use std::{thread, sync::{mpsc::{self}, Arc, Mutex}};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
} 

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool.
    /// 
    /// # Panics
    /// 
    /// The 'new' function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, reciever) = mpsc::channel();

        let reciever = Arc::new(Mutex::new(reciever));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(
                id, 
       Arc::clone(&reciever)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
        {
            let job = Box::new(f);
            match self.sender.send(Message::NewJob(job)) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Workers stopped running unexpectedly: {}, SHUTTING DOWN.", e); 
                    std::process::exit(1)
                }
            };
        }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        debug!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        debug!("Shutting down all workers.");

        for worker in &mut self.workers {
            debug!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
} 

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = reciever
                .lock()
                .unwrap()
                .recv()
                .unwrap();

            match message {
                Message::NewJob(job) => {
                    debug!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    debug!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}