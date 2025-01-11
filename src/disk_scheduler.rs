use std::future::Future;
use std::marker::PhantomData;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use crate::disk_manager::{DiskManager, DiskRequest};

/// [DiskScheduler] implements a IO scheduler for reading and writing
/// from disk in to memory.
pub struct DiskScheduler {
    /// channel is used a queue for disk reads and writes to be processed
    channel: (Sender<Option<DiskRequest>>, Receiver<Option<DiskRequest>>),
    /// disk_manager
    disk_manager: DiskManager,
}

impl DiskScheduler {
    pub fn new(disk_manager: DiskManager) -> Self {
        let channel: (Sender<Option<DiskRequest>>, Receiver<Option<DiskRequest>>) = mpsc::channel();
        Self {
            channel,
            disk_manager,
        }
    }

    pub fn spawn_worker(&self) -> std::thread::JoinHandle<()> {
        let (tx, rw) = self.channel.clone();
        std::thread::spawn(|| {
            while let Ok(r) = rw.recv() {
                match r {
                    None => {}
                    Some(disk_request) => {
                        if disk_request.is_write {
                            self.disk_manager.write_page(disk_request);
                        } else {
                            self.disk_manager.write_page(disk_request);
                        }
                    }
                }
            }
        })
    }

    pub fn schedule_io(&self, disk_request: DiskRequest) {
        self.channel.0.send(Some(disk_request)).unwrap();
    }
}
