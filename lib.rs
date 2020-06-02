use std::thread;
use std::slice::Join;
use std::thread::JoinHandle;
use std::sync::{mpsc, Mutex, Arc};

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

type Job = Box<dyn FnBox + Send + 'static>;

impl ThreadPool{
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size{
            workers.push(worker::new(id,Arc::clone(&receiver)))
        }

        Ok(ThreadPool{
            workers,
            sender
        })
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();

    }


}


struct Worker{
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker{
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker{
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job: executing.", id);
                        job.call_box();
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);
                        break;
                    }
                }

            }
        });
        Worker{
            id,
            thread: Some(thread)
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self){
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            for _ in &mut self.workers {
                self.sender.send(Message::Terminate).unwrap();
            }

            if let Some(thread) = worker.thread.take(){
                thread.join().unwrao();
            }
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate
}

