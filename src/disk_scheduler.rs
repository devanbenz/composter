use crate::disk_manager::{DiskManager, DiskManagerRequest, DiskRequest};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

/// [DiskScheduler] implements a IO scheduler for reading and writing
/// from disk in to memory.
pub struct DiskScheduler {
    /// channel is used a queue for disk reads and writes to be processed
    sender: Sender<DiskManagerRequest>,
}

impl DiskScheduler {
    pub fn new(disk_manager: DiskManager) -> Self {
        let (tx, rx) = mpsc::channel();
        Self::spawn_worker(rx, disk_manager);
        Self { sender: tx }
    }

    pub fn spawn_worker(
        receiver: Receiver<DiskManagerRequest>,
        mut disk_manager: DiskManager,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || loop {
            match receiver.try_recv() {
                Ok(req) => match req {
                    DiskManagerRequest::DiskRwRequest(req) => {
                        if req.is_write {
                            disk_manager.write_page(req);
                        } else {
                            disk_manager.read_page(req);
                        }
                    }
                    DiskManagerRequest::DiskIncreaseRequest(size) => {}
                    DiskManagerRequest::DiskDecreaseRequest(size) => {}
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

    pub fn increase_size(
        &mut self,
        size: usize,
    ) -> Result<(), mpsc::SendError<DiskManagerRequest>> {
        self.sender
            .send(DiskManagerRequest::DiskIncreaseRequest(size))
    }

    pub fn decrease_size(
        &mut self,
        size: usize,
    ) -> Result<(), mpsc::SendError<DiskManagerRequest>> {
        self.sender
            .send(DiskManagerRequest::DiskDecreaseRequest(size))
    }

    pub fn request(&self, request: DiskRequest) -> Result<(), mpsc::SendError<DiskManagerRequest>> {
        self.sender.send(DiskManagerRequest::DiskRwRequest(request))
    }
}

mod tests {
    use super::*;
    use std::path::PathBuf;
    #[test]
    fn test_disk_scheduler() {
        let mut counter = 0;
        let (tx, rx) = mpsc::channel();
        let dm = DiskManager::new(4096, None, true);
        let ds = DiskScheduler::new(dm);

        let b = std::thread::spawn(move || {
            for i in 0..10 {
                ds.request(DiskRequest {
                    is_write: false,
                    data: vec![],
                    page_id: i,
                    callback: tx.clone(),
                })
                .unwrap();
            }
        });

        let a = std::thread::spawn(move || {
            while let Ok(val) = rx.recv() {
                assert_eq!(val, true);
                counter = counter + 1;
            }
        });

        b.join().unwrap();
        a.join().unwrap();

        assert_eq!(counter, 10);
    }
}
