pub mod threadlib {
    use std::thread;
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::sync::Mutex;
    
    pub enum Message {
        NewJob(Job),
        Terminate,
    }
    
    pub struct ThreadPool{
        threads: Vec<Worker>,
        sender: mpsc::Sender<Message>
    }
    
    pub struct Worker {
        id: usize,
        handle: Option<thread::JoinHandle<()>>
    }
    
    type Job = Box<dyn FnBox + Send + 'static>;
    
    pub trait FnBox {
        fn call_box(self: Box<Self>);
    }
    
    impl<F: FnOnce()> FnBox for F {
        fn call_box(self:Box<F>) {
            (*self)()
        }
    }
    
    impl Worker {
        pub fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
           Worker { id, handle: Some(thread::spawn(move ||{
                loop{
                    let message = reciever.lock().unwrap().recv().unwrap();
    
                    match message {
                        Message::NewJob(job) => {
                            println!("Worker {} got a job; executing.", id);
                            job.call_box();
                        },
                        Message::Terminate => {
                            println!("Worker {} was told to terminate.", id);
                            break;
                        },
                    }
                }
           }))} 
        }
    }
    
    impl Drop for ThreadPool {
        fn drop(&mut self) {
            println!("Senging terminate message to all workers.");
            for _ in &mut self.threads {
                self.sender.send(Message::Terminate).unwrap();
            }
            println!("Shutting down all workers.");
            for worker in &mut self.threads {
                println!("Shutting down worker {}", worker.id);
                if let Some(thread) = worker.handle.take() {
                    thread.join().unwrap();
                }
            }
        }
    }
    impl ThreadPool {
        pub fn new(size:usize) -> ThreadPool {
            assert!(size > 0);
            let (sender, reciever) = mpsc::channel();
            let reciever = Arc::new(Mutex::new(reciever));
            let mut threads = Vec::with_capacity(size);
    
            for i in 0..size {
                threads.push(Worker::new(i, Arc::clone(&reciever)));
            }
    
            ThreadPool {
                threads,    
                sender
            }
        }
    
        pub fn execute<F>(&mut self, f: F)
            where
                F: FnOnce() + Send + 'static
        {
            let job = Box::new(f);
            self.sender.send(Message::NewJob(job)).unwrap();
        }
    }
}
