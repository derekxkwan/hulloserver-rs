use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// create new threadpool
    ///
    /// size = num of threads
    ///
    /// panics if size <= 0
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (tx, rx) = mpsc::channel();

        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }

        ThreadPool{workers, sender: tx}
    }

    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
        {
            let job = Box::new(f);

            self.sender.send(Message::NewJob(job)).unwrap();
        }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {

        println!("Telling all workers to quit");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Making workers quit");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn( move || loop {

            let msg = rx.lock().unwrap().recv().unwrap();

            match msg {
                Message::NewJob(job) => {
                    println!("I am worker {} and I got a job!", id);

                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to quit.", id)

                }
            }
        });

        Worker {id, thread: Some(thread)}
    }
}


