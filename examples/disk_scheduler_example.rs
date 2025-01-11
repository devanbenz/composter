use composter::disk_scheduler::DiskScheduler;
use std::ptr::write;

#[tokio::main]
async fn main() {
    let fut_fn = || async { true };
    let ds = DiskScheduler::new(fut_fn);

    let handles = vec![
        ds.spawn_worker(),
        ds.spawn_worker(),
        ds.spawn_worker(),
        ds.spawn_worker(),
    ];

    for handle in handles {
        handle.join().unwrap();
    }
}
