use crate::clock_replacer::Replacer;
use crate::disk_manager::DiskManager;
use crate::disk_scheduler::DiskScheduler;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

struct Frame {
    buffer: Vec<u8>,
    pin_count: AtomicU64,
    current_page_index: usize,
    
}

struct BufferPoolManager {
    disk_manager: DiskManager,
    disk_scheduler: DiskScheduler,
    page_table: HashMap<usize, usize>,
    free_list: Vec<usize>,
    current_page_index: AtomicU64,
    replacer: Replacer<usize>,
    frames: Vec<Frame>,
}

impl BufferPoolManager {
    pub fn new(
        disk_manager: DiskManager,
        disk_scheduler: DiskScheduler,
        replacer: Replacer<usize>,
        num_frames: usize,
    ) -> BufferPoolManager {
        let page_table = HashMap::new();
        let free_list = Vec::new();
        let current_page_index = AtomicU64::new(0);
        let frames = Vec::new();

        BufferPoolManager {
            disk_manager,
            disk_scheduler,
            page_table,
            free_list,
            current_page_index,
            replacer,
            frames,
        }
    }
    
    pub fn checked_read_page(page_id: usize) -> Option<>
}
