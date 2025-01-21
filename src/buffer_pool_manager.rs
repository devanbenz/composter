use crate::clock_replacer::Replacer;
use crate::disk_manager::DiskManager;
use crate::disk_scheduler::DiskScheduler;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU64, AtomicUsize};

struct Frame {
    buffer: Vec<u8>,
    pin_count: AtomicU64,
    current_page_index: Option<usize>,
}

impl Frame {
    fn new(page_size: usize) -> Self {
        let buffer = vec![0; page_size];
        Self {
            buffer,
            pin_count: AtomicU64::new(0),
            current_page_index: None,
        }
    }
}

struct BufferPoolManager {
    disk_scheduler: DiskScheduler,
    page_table: HashMap<usize, usize>,
    free_list: Vec<usize>,
    current_page_index: AtomicUsize,
    replacer: Replacer<usize>,
    frames: Vec<Frame>,
    page_size: usize,
}

impl BufferPoolManager {
    pub fn new(
        disk_scheduler: DiskScheduler,
        replacer: Replacer<usize>,
        page_size: usize,
        num_frames: usize,
    ) -> BufferPoolManager {
        let page_table = HashMap::new();
        let current_page_index = AtomicUsize::new(0);
        let frames = (0..num_frames).map(|_| Frame::new(page_size)).collect();
        let free_list = (0..num_frames).collect();

        BufferPoolManager {
            disk_scheduler,
            page_table,
            free_list,
            current_page_index,
            replacer,
            frames,
            page_size,
        }
    }

    /// new_page creates a new page entry on disk
    /// increasing the file size of the page file
    /// it returns the page_id
    pub fn new_page(&mut self) -> usize {
        self.current_page_index.fetch_add(1, Relaxed);
        self.disk_scheduler.new_page(self.page_size);

        self.current_page_index.load(Relaxed)
    }

    pub fn read() {}

    pub fn write() {}
}

mod tests {
    use super::*;
    use crate::DEFAULT_PAGE_SIZE;
    use std::sync::{Arc, Mutex};
    #[test]
    fn new_buffer_pool_manager() {
        let disk_manager = Arc::new(Mutex::new(DiskManager::default()));
        let disk_scheduler = DiskScheduler::new(disk_manager);
        let replacer = Replacer::new(10);
        let buffer_pool_manager =
            BufferPoolManager::new(disk_scheduler, replacer, DEFAULT_PAGE_SIZE, 10);

        assert_eq!(buffer_pool_manager.page_table.len(), 0);
        assert_eq!(buffer_pool_manager.free_list.len(), 10);
        assert_eq!(buffer_pool_manager.frames.len(), 10);
    }

    #[test]
    fn add_pages() {
        let disk_manager = Arc::new(Mutex::new(DiskManager::default()));
        let disk_scheduler = DiskScheduler::new(disk_manager);
        let replacer = Replacer::new(10);
        let mut buffer_pool_manager =
            BufferPoolManager::new(disk_scheduler, replacer, DEFAULT_PAGE_SIZE, 10);

        assert_eq!(buffer_pool_manager.page_table.len(), 0);
        assert_eq!(buffer_pool_manager.free_list.len(), 10);
        assert_eq!(buffer_pool_manager.frames.len(), 10);

        // Create 10 new pages in memory for this test
        // return the page index.
        for i in 1..11 {
            let np = buffer_pool_manager.new_page();
            assert_eq!(np, i);
        }
    }
}
