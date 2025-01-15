use crate::clock_replacer::Replacer;
use crate::disk_manager::DiskManager;
use crate::disk_scheduler::DiskScheduler;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

struct Frame {
    buffer: Vec<u8>,
    pin_count: AtomicU64,
    current_page_index: Option<usize>,
}

impl Frame {
    fn new(page_size: usize) -> Self {
        let buffer = vec![0; page_size];
        Self { buffer, pin_count: AtomicU64::new(0), current_page_index: None }
    }
}

struct BufferPoolManager {
    disk_scheduler: DiskScheduler,
    page_table: HashMap<usize, usize>,
    free_list: Vec<usize>,
    current_page_index: AtomicU64,
    replacer: Replacer<usize>,
    frames: Vec<Frame>,
}

impl BufferPoolManager {
    pub fn new(
        disk_scheduler: DiskScheduler,
        replacer: Replacer<usize>,
        page_size: usize,
        num_frames: usize,
    ) -> BufferPoolManager {
        let page_table = HashMap::new();
        let current_page_index = AtomicU64::new(0);
        let frames = (0..num_frames).map(|_| Frame::new(page_size)).collect();
        let free_list = (0..num_frames).map(|v| v).collect();

        BufferPoolManager {
            disk_scheduler,
            page_table,
            free_list,
            current_page_index,
            replacer,
            frames,
        }
    }

    pub fn read_page() {
        unimplemented!()
    }

    pub fn write_page() {
        unimplemented!()
    }
}

mod tests {
    use crate::DEFAULT_PAGE_SIZE;
    use super::*;
    #[test]
    fn new_buffer_pool_manager() {
        let disk_manager = DiskManager::default();
        let disk_scheduler = DiskScheduler::new(disk_manager);
        let replacer = Replacer::new(10);
        let buffer_pool_manager = BufferPoolManager::new(disk_scheduler, replacer, DEFAULT_PAGE_SIZE, 10);

        assert_eq!(buffer_pool_manager.page_table.len(), 0);
        assert_eq!(buffer_pool_manager.free_list.len(), 10);
        assert_eq!(buffer_pool_manager.frames.len(), 10);
    }
}
