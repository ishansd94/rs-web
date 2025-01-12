use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use logger;

type Job = Box<dyn FnOnce() + Send +'static>;
enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {

        assert!(size > 0);

        logger::info(format!("Creating thread pool with {} workers", size).as_str());

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send +'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    
    fn drop(&mut self) {

        logger::warn("Sending terminate signal to all workers...");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        logger::warn("Waiting for all workers to finish...");

        for worker in &mut self.workers {

            logger::warn(format!("Shutting down worker {}...", worker.id).as_str());
            
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
    
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            match job {
                Message::NewJob(job) => {
                    logger::debug(format!("Worker {} executing job.", id).as_str());
                    job();
                }
                Message::Terminate => {
                    logger::warn(format!("Worker {} received terminate signal.", id).as_str());
                    break;
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}