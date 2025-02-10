use crate::clock_replacer::{Evictable, Replacer};
use crate::disk_manager::DiskManager;
use crate::disk_scheduler::DiskScheduler;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::{Arc, LockResult, Mutex, RwLock};

enum CheckedPage {
    Read(ReadPage),
    Write(WritePage),
}

pub struct ReadPage {
    pub page_id: usize,
    pub pinned: AtomicUsize,
    pub frame: Arc<Mutex<Frame>>,
}

impl ReadPage {
    pub fn new(page_id: usize, pinned: AtomicUsize, frame: Arc<Mutex<Frame>>) -> Self {
        Self {
            page_id,
            pinned,
            frame,
        }
    }

    pub fn is_dirty(&self) -> Result<bool, std::io::Error> {
        let frame = self.frame.lock();
        match frame {
            Ok(frame) => Ok(frame.dirty),
            Err(err) => Err(std::io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }
}

impl Read for ReadPage {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let frame = self.frame.lock();
        match frame {
            Ok(frame) => {
                let frame_buf = &frame.buffer;
                buf.copy_from_slice(frame_buf);
                Ok(buf.len())
            }
            Err(err) => Err(std::io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }
}

impl Drop for ReadPage {
    fn drop(&mut self) {
        let frame = self.frame.lock();
        match frame {
            Ok(frame) => {
                frame.pin_count.fetch_sub(1, Relaxed);
            }
            Err(err) => {
                eprintln!("Error occurred during drop: {}", err);
            }
        }
    }
}

pub struct WritePage {
    pub page_id: usize,
    pub pinned: AtomicUsize,
    pub frame: Arc<Mutex<Frame>>,
}

impl WritePage {
    pub fn new(page_id: usize, pinned: AtomicUsize, frame: Arc<Mutex<Frame>>) -> Self {
        Self {
            page_id,
            pinned,
            frame,
        }
    }

    pub fn is_dirty(&self) -> Result<bool, std::io::Error> {
        let frame = self.frame.lock();
        match frame {
            Ok(frame) => Ok(frame.dirty),
            Err(err) => Err(std::io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }
}

impl Write for WritePage {
    fn write(&mut self, data: &[u8]) -> Result<usize, std::io::Error> {
        let frame = &mut self.frame.lock();
        match frame {
            Ok(frame) => {
                frame.dirty = true;
                frame.write(data)
            }
            Err(err) => Err(std::io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unimplemented!()
    }
}

impl Drop for WritePage {
    fn drop(&mut self) {
        let frame = &mut self.frame.lock();
        match frame {
            Ok(frame) => {
                frame.pin_count.fetch_sub(1, Relaxed);
            }
            Err(err) => {
                panic!("Error occurred during drop: {}", err);
            }
        };
    }
}

#[derive(Clone)]
struct ReplacerNode {
    frame_id: usize,
    evictable: bool,
}

impl Evictable<ReplacerNode> for ReplacerNode {
    fn new(id: usize) -> ReplacerNode {
        ReplacerNode {
            frame_id: id,
            evictable: true,
        }
    }

    fn pinned(&self) -> bool {
        self.evictable
    }

    fn id(&self) -> usize {
        self.frame_id
    }
}

pub struct Frame {
    pub buffer: Vec<u8>,
    pub pin_count: AtomicU64,
    pub current_page_index: Option<usize>,
    pub page_size: usize,
    pub dirty: bool,
}

impl Write for Frame {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.len() > self.page_size {
            return Err(std::io::Error::new(
                ErrorKind::OutOfMemory,
                "buffer is too large",
            ));
        }
        let curr_len = &self
            .buffer
            .iter()
            .take_while(|x| *x != &0)
            .collect::<Vec<_>>();
        let curr_len = curr_len.len();
        let total_len = curr_len + buf.len();
        if total_len > self.page_size {
            return Err(std::io::Error::new(
                ErrorKind::OutOfMemory,
                "buffer will exceed page size",
            ));
        }

        self.buffer[curr_len..total_len].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unimplemented!()
    }
}

impl Frame {
    fn new(page_size: usize) -> Self {
        let buffer = vec![0; page_size];
        Self {
            buffer,
            pin_count: AtomicU64::new(0),
            current_page_index: None,
            page_size,
            dirty: false,
        }
    }
}

struct BufferPoolManager {
    disk_scheduler: DiskScheduler,
    page_table: Arc<Mutex<HashMap<usize, usize>>>,
    free_list: Arc<Mutex<Vec<usize>>>,
    current_page_index: AtomicUsize,
    replacer: Arc<Mutex<Replacer<ReplacerNode>>>,
    frames: Vec<Arc<Mutex<Frame>>>,
    page_size: usize,
}

impl BufferPoolManager {
    pub fn new(
        disk_scheduler: DiskScheduler,
        replacer: Replacer<ReplacerNode>,
        page_size: usize,
        num_frames: usize,
    ) -> BufferPoolManager {
        let page_table = HashMap::new();
        let current_page_index = AtomicUsize::new(0);
        let frames = (0..num_frames)
            .map(|_| Arc::new(Mutex::new(Frame::new(page_size))))
            .collect::<Vec<_>>();
        let free_list = (0..num_frames).collect();

        BufferPoolManager {
            disk_scheduler,
            page_table: Arc::new(Mutex::new(page_table)),
            free_list: Arc::new(Mutex::new(free_list)),
            current_page_index,
            replacer: Arc::new(Mutex::new(replacer)),
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

    pub fn read_page(&self, page_id: usize) -> Option<ReadPage> {
        if let Some(frame_id) = self.check_page(page_id) {
            if let Some(frame) = self.frames.get(frame_id) {
                let frame_copy = Arc::clone(frame);
                Some(ReadPage {
                    page_id,
                    pinned: AtomicUsize::new(1),
                    frame: frame_copy,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn write_page(&self, page_id: usize) -> Option<WritePage> {
        if let Some(frame_id) = self.check_page(page_id) {
            if let Some(frame) = self.frames.get(frame_id) {
                let frame_copy = Arc::clone(frame);
                Some(WritePage {
                    page_id,
                    pinned: AtomicUsize::new(1),
                    frame: frame_copy,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// check_page checks if the requests page
    /// is already mapped to a frame. If it is not
    /// eviction can occur and a frame will be freed
    /// which then a new page will be brought in.  
    fn check_page(&self, page_id: usize) -> Option<usize> {
        if page_id > self.current_page_index.load(Relaxed) {
            return None;
        }

        let mut page_table = self.page_table.try_lock().unwrap();
        let mut free_list = self.free_list.try_lock().unwrap();
        let mut replacer = self.replacer.try_lock().unwrap();

        match page_table.get(&page_id) {
            Some(frame_id) => Some(*frame_id),
            None => {
                let free_frame = free_list.get(page_id);
                if free_frame.is_some() {
                    return free_frame.copied();
                }

                let evicted_frame = replacer
                    .insert_and_evict(page_id)
                    .expect("failed to evict page");
                if let Some(item) = evicted_frame {
                    free_list.push(item.id());
                    page_table.remove(&page_id);
                }
                None
            }
        }
    }
}

mod tests {
    use super::*;
    use crate::DEFAULT_PAGE_SIZE;
    use std::cell::RefCell;
    use std::io::Error;
    use std::sync::{Arc, Mutex};
    #[test]
    fn new_buffer_pool_manager() {
        let disk_manager = Arc::new(Mutex::new(DiskManager::default()));
        let disk_scheduler = DiskScheduler::new(disk_manager);
        let replacer = Replacer::new(10);
        let buffer_pool_manager =
            BufferPoolManager::new(disk_scheduler, replacer, DEFAULT_PAGE_SIZE, 10);

        assert_eq!(buffer_pool_manager.frames.len(), 10);
    }

    #[test]
    fn add_pages() {
        let disk_manager = Arc::new(Mutex::new(DiskManager::default()));
        let disk_scheduler = DiskScheduler::new(disk_manager);
        let replacer = Replacer::new(10);
        let mut buffer_pool_manager =
            BufferPoolManager::new(disk_scheduler, replacer, DEFAULT_PAGE_SIZE, 10);

        assert_eq!(buffer_pool_manager.frames.len(), 10);

        // Create 10 new pages in memory for this test
        // return the page index.
        for i in 1..11 {
            let np = buffer_pool_manager.new_page();
            assert_eq!(np, i);
        }
    }

    #[test]
    fn test_frame_writer() {
        let mut frame = Frame::new(5);
        assert_eq!(vec![0, 0, 0, 0, 0], frame.buffer);
        let val = frame.write(&[1, 2, 3]).unwrap();
        assert_eq!(val, 3);
        assert_eq!(vec![1, 2, 3, 0, 0], frame.buffer);
        let val = frame.write(&[4]).unwrap();
        assert_eq!(val, 1);
        assert_eq!(vec![1, 2, 3, 4, 0], frame.buffer);
        let val = frame.write(&[5]).unwrap();
        assert_eq!(val, 1);
        assert_eq!(vec![1, 2, 3, 4, 5], frame.buffer);
        match frame.write(&[6]) {
            Ok(_val) => {
                panic!("should not have a value")
            }
            Err(err) => {
                assert_eq!(err.to_string(), "buffer will exceed page size");
            }
        }
    }

    #[test]
    fn test_read_write_page_frame() {
        let mut frame = Arc::new(Mutex::new(Frame::new(5)));
        let f = Arc::clone(&frame);
        assert!(!f.lock().unwrap().dirty);

        let mut wp = WritePage {
            page_id: 1,
            pinned: Default::default(),
            frame: f,
        };

        let a = wp.write(&[97, 97]).unwrap();
        assert_eq!(a, 2);
        let is_dirty = wp.is_dirty().unwrap();
        assert!(is_dirty);

        drop(wp);

        let mut rp = ReadPage {
            page_id: 1,
            pinned: Default::default(),
            frame,
        };

        let mut buf = vec![0; 5];
        let buf_read = rp.read(&mut buf);
        assert!(buf_read.is_ok());
        assert_eq!(buf_read.unwrap(), 5);
        assert_eq!(buf, vec![97, 97, 0, 0, 0]);
    }

    #[test]
    fn test_read_write_page() {
        let disk_manager = Arc::new(Mutex::new(DiskManager::default()));
        let disk_scheduler = DiskScheduler::new(disk_manager);
        let replacer = Replacer::new(10);
        let mut buffer_pool_manager =
            BufferPoolManager::new(disk_scheduler, replacer, DEFAULT_PAGE_SIZE, 10);

        let np = buffer_pool_manager.new_page();
        assert_eq!(np, 1);
        let np2 = buffer_pool_manager.new_page();
        assert_eq!(np2, 2);
        let rp = buffer_pool_manager.read_page(np);
        assert!(rp.is_some());
        let rp2 = buffer_pool_manager.read_page(10);
        assert!(rp2.is_none());

        // Write to a page and then read from it
        let wp = buffer_pool_manager.write_page(np);
        assert!(wp.is_some());

        match wp.unwrap().write(b"foo") {
            Ok(written) => {
                assert_eq!(written, 3);
            }
            Err(err) => {
                panic!("failed to write buffer page: {}", err);
            }
        }

        let rp = buffer_pool_manager.read_page(np);
        assert!(rp.is_some());
        let mut buf = [0_u8; DEFAULT_PAGE_SIZE];

        match rp.unwrap().read(&mut buf) {
            Ok(_written) => {}
            Err(err) => {
                panic!("failed to write buffer page: {}", err);
            }
        }

        assert_eq!(buf[..3], [102, 111, 111]);

        let wp = buffer_pool_manager.write_page(np);
        assert!(wp.is_some());

        match wp.unwrap().write(b"bar") {
            Ok(written) => {
                assert_eq!(written, 3);
            }
            Err(err) => {
                panic!("failed to write buffer page: {}", err);
            }
        }

        let rp = buffer_pool_manager.read_page(np);
        assert!(rp.is_some());
        let mut buf = [0_u8; DEFAULT_PAGE_SIZE];

        match rp.unwrap().read(&mut buf) {
            Ok(_written) => {}
            Err(err) => {
                panic!("failed to write buffer page: {}", err);
            }
        }

        assert_eq!(buf[..6], [102, 111, 111, 98, 97, 114]);
    }

    #[test]
    fn test_page_contention() {}
}
