use crate::clock_replacer::Replacer;
use crate::disk_manager::DiskManager;
use crate::disk_scheduler::DiskScheduler;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

struct BufferPoolManager {
    disk_manager: DiskManager,
    disk_scheduler: DiskScheduler,
    page_table: HashMap<usize, usize>,
    free_list: Vec<usize>,
    current_page_index: AtomicU64,
    replacer: Replacer<usize>,
}
