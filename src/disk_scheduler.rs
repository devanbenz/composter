use crate::disk_manager::{DiskManager, DiskManagerRequest};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{mpsc, Arc, Mutex};

/// [DiskScheduler] implements a IO scheduler for reading and writing
/// from disk in to memory.
pub struct DiskScheduler {
    /// channel is used a queue for disk reads and writes to be processed
    sender: Sender<DiskManagerRequest>,
    disk_manager: Arc<Mutex<DiskManager>>,
}

impl DiskScheduler {
    pub fn new(disk_manager: Arc<Mutex<DiskManager>>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self::spawn_worker(rx, disk_manager.clone());

        Self {
            sender: tx,
            disk_manager,
        }
    }

    pub fn spawn_worker(
        receiver: Receiver<DiskManagerRequest>,
        disk_manager: Arc<Mutex<DiskManager>>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || loop {
            match receiver.try_recv() {
                Ok(req) => match req {
                    DiskManagerRequest::DiskRwRequest {
                        is_write,
                        data,
                        page_id,
                        callback,
                    } => {
                        let mut data = data.lock().unwrap();
                        let mut_data = data.as_mut();
                        let mut dm = disk_manager.lock().unwrap();
                        if is_write {
                            dm.write_page(mut_data, page_id, callback);
                        } else {
                            dm.read_page(mut_data, page_id, callback);
                        }
                    }
                },
                Err(TryRecvError::Empty) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(TryRecvError::Disconnected) => {
                    break;
                }
            }
        })
    }

    pub fn new_page(&mut self, size: usize) {
        let mut dm = self.disk_manager.lock().unwrap();
        dm.increase_pages(size);
    }

    pub fn request(
        &mut self,
        is_write: bool,
        data: Arc<Mutex<Vec<u8>>>,
        page_id: usize,
        callback: Sender<bool>,
    ) -> Result<(), mpsc::SendError<DiskManagerRequest>> {
        self.sender.send(DiskManagerRequest::DiskRwRequest {
            is_write,
            data,
            page_id,
            callback,
        })
    }
}

mod tests {
    use super::*;
    #[test]
    fn test_disk_scheduler() {
        let (call_tx, call_rx) = mpsc::channel();
        let dm = Arc::new(Mutex::new(DiskManager::new(4096, None, true)));
        let mut ds = DiskScheduler::new(dm);

        let b = std::thread::spawn(move || {
            for i in 1..10 {
                let v = Arc::new(Mutex::new(vec![0; 1024]));
                ds.new_page(i);
                ds.request(false, v, i, call_tx.clone()).unwrap();
            }
        });

        let a = std::thread::spawn(move || {
            while let Ok(val) = call_rx.recv() {
                assert!(val);
                println!("ok");
            }
        });

        b.join().unwrap();
        a.join().unwrap();
    }
}
