use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::mpsc::{Receiver, Sender};
use std::task::{Context, Poll};

/// [DiskScheduler] implements a IO scheduler for reading and writing
/// from disk in to memory.
pub struct DiskScheduler<T, Fut, R> {
    /// fut_fn is a future for dealing with the return value of the scheduled disk io
    fut_fn: T,
    channel: (Receiver<Option<R>>, Sender<Option<R>>),
    f: PhantomData<Fut>,
    r: PhantomData<R>,
}

impl<T, Fut, R> DiskScheduler<T, Fut, R>
where
    T: FnOnce() -> Fut,
    Fut: Future<Output = R> + Send + 'static,
{
    pub fn new(fut_fn: T) -> Self {
        Self {
            fut_fn,
            f: Default::default(),
            r: Default::default(),
        }
    }

    pub fn spawn_worker(&self) -> std::thread::JoinHandle<()> {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let txt = format!(
                "[{:?}] worker thread running...",
                std::thread::current().id()
            );
            std::println!("{}", txt);
            std::thread::sleep(std::time::Duration::from_secs(1));
            std::println!("worker thread exiting...");
        })
    }
    
    pub fn 
}
