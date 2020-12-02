use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::error::Error;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate
}

/// Custom ThreadPool that uses channels to deliver work items to threads.
///
/// ```
/// fn main() {
/// use web_server::ThreadPool;
///
/// let pool = ThreadPool::new(5);
/// pool.execute(|| {
/// println!("This task was submitted to the threadpool!");
/// })
/// }
/// ```
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        //assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Method to submit a job to the thread pool
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
    {
        self.sender.send(Message::NewJob(Box::new(f)))
            .expect("Oh noes! Looks like none of the threads in the thread pool are alive!");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate)
                .expect("Failed to termination message to the one or more of the workers!");
        }

        println!("Shutting down all workers!");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("Failed to join on one of the worker threads!");
            }
        }
    }
}

/// Thread pool worker, encapsulating an id and a thread `JoinHandle`
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Worker initialization method
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                },
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread)
        }
    }
}
#[cfg(test)]
pub mod tests {
    #[test]
    pub fn test_function() {
        println!("Running a dummy test! WEEEEEE!!!!");
    }

}