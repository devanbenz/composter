use composter::disk_scheduler::DiskScheduler;
use std::ptr::write;
use composter::disk_manager::{DiskManager, DiskRequest};

#[tokio::main]
async fn main() {
    let fut_fn = || async { true };
    let dm = DiskManager {};
    let ds = DiskScheduler::new(dm);

    let handles = vec![
        ds.spawn_worker(),
        ds.spawn_worker(),
        ds.spawn_worker(),
        ds.spawn_worker(),
    ];

    for handle in handles {
        handle.join().unwrap();
    }

    ds.schedule_io(DiskRequest {
        is_write: false,
        data: vec![],
        page_id: 0,
        callback: None,
    })
}
