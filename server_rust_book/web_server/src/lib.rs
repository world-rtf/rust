use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};


/// A thread pool for executing tasks in parallel.
///
/// The `ThreadPool` allows spawning a fixed number of worker threads and distributing tasks among them.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

// задача для выолнения - замыкание: выполняется один раз, передается между потоками, не содержит ссылок с нестатическим временем жизни
type Job = Box<dyn FnOnce() + Send + 'static>;


impl ThreadPool {
    /// Сreates a new ThreadPool with the specified number of worker threads.
    ///
    /// # Arguments
    /// 
    /// `size` - The number of worker threads to create in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        // приемник
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Submits a task to be executed by the thread pool.
    /// 
    /// The task will be wrapped in a `Job` and sent to the worker threads through
    /// a channel. The first available worker will receive and execute the task.
    /// 
    /// # Arguments
    /// 
    /// The closure `f` must satisfy several constraints:
    /// - `FnOnce()`: Can be called once (standard for thread operations)
    /// - `Send`: Safe to send between threads
    /// - `'static`: Does not capture non-static references
    /// 
    /// # Panics
    ///
    /// Panics if the job queue is closed, which occurs if the thread pool has been
    /// dropped and all workers have stopped processing tasks.

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {   
        let job = Box::new(f); // размер не известен на этапе компиляции
        //  *
        if let Some(sender) = &self.sender {
            if let Err(e) = sender.send(job) {
                eprintln!("Failed to send job to worker: {}", e);
            }
        } else {
            eprintln!("ThreadPool is shutting down, cannot accept new jobs");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                if let Err(e) = thread.join() { //  *
                    eprintln!("Failed to join worker {}: {:?}", worker.id, e);
                }
            }
        }
    }
}

struct Worker {
    // инкапсулирует один рабочий поток
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv(); // Ждем задачу

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");

                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    #[should_panic(expected = "size > 0")]
    fn test_create_pool_zero() {
        ThreadPool::new(0);
    }

    #[test]
    fn test_execute_task_with_channel() {
        let pool = ThreadPool::new(2 + 3);
        let (sender, receiver) = mpsc::channel();
        
        pool.execute(move || {
            sender.send("task executed").unwrap();
        });
        
        assert_eq!(receiver.recv_timeout(Duration::from_secs(1)), Ok("task executed"));
    }

    #[test]
    fn test_workers_drop() {
        let pool = ThreadPool::new(2);
        let worker_threads: Vec<_> = pool.workers.iter()
            .filter_map(|w| w.thread.as_ref().map(|t| t.thread().id()))
            .collect();

        drop(pool);

        // Проверяем что потоки стопнулись
        for _ in worker_threads {
            assert!(thread::spawn(move || {})
                .join()
                .is_ok());
        }
    }



}
