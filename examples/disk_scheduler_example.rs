use composter::disk_manager::{DiskManager, DiskRequest};
use composter::disk_scheduler::DiskScheduler;
use std::path::PathBuf;
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    let dm = DiskManager::new(
        4096,
        Some(PathBuf::from(
            "/Users/devan/Documents/Projects/composter/scratch_page/sample.db",
        )),
        false,
    );
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
            println!("{:?}", val);
        }
    });

    b.join().unwrap();
    a.join().unwrap();
}
