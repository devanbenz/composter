use crate::clock_replacer::{Evictable, Replacer};
use crate::disk_manager::DiskManager;
use crate::disk_scheduler::DiskScheduler;
use std::collections::HashMap;
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::{Arc, LockResult, Mutex, MutexGuard, PoisonError};

enum CheckedPage<'a> {
    Read(ReadPage<'a>),
    Write(WritePage<'a>),
}

pub struct ReadPage<'a> {
    pub page_id: usize,
    pub pinned: AtomicUsize,
    pub frame: Arc<Mutex<&'a Frame>>,
}

impl<'a> ReadPage<'a> {
    pub fn read(&self) -> &'a [u8] {
        let frame = self.frame.lock();
        frame.unwrap().buffer.deref()
    }
}

pub struct WritePage<'a> {
    pub page_id: usize,
    pub pinned: AtomicUsize,
    pub frame: Arc<Mutex<&'a mut Frame>>,
}

impl<'a> Write for WritePage<'a> {
    fn write(&mut self, data: &[u8]) -> Result<usize, std::io::Error> {
        let frame = &mut self.frame.lock();
        match frame {
            Ok(frame) => frame.write(data),
            Err(err) => Err(std::io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
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
        todo!()
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
        }
    }
}

struct BufferPoolManager {
    disk_scheduler: DiskScheduler,
    page_table: HashMap<usize, usize>,
    free_list: Vec<usize>,
    current_page_index: AtomicUsize,
    replacer: Replacer<ReplacerNode>,
    frames: Vec<Frame>,
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

    pub fn read_page(&mut self, page_id: usize) -> Option<ReadPage> {
        if let Some(frame_id) = self.check_page(page_id) {
            let frame_ = &self.frames[frame_id];
            let wrapped_frame = Arc::new(Mutex::new(frame_));
            Some(ReadPage {
                page_id,
                pinned: AtomicUsize::new(1),
                frame: wrapped_frame,
            })
        } else {
            None
        }
    }

    pub fn write_page(&mut self, page_id: usize) -> Option<WritePage> {
        if let Some(frame_id) = self.check_page(page_id) {
            let frame_ = &mut self.frames[frame_id];
            let wrapped_frame = Arc::new(Mutex::new(frame_));
            Some(WritePage {
                page_id,
                pinned: AtomicUsize::new(1),
                frame: wrapped_frame,
            })
        } else {
            None
        }
    }

    /// check_page checks if the requests page
    /// is already mapped to a frame. If it is not
    /// eviction can occur and a frame will be freed
    /// which then a new page will be brought in.  
    fn check_page(&mut self, page_id: usize) -> Option<usize> {
        match self.page_table.get(&page_id) {
            Some(frame_id) => Some(*frame_id),
            None => {
                let free_frame = self.free_list.pop();
                if free_frame.is_some() {
                    return free_frame;
                }
                let evicted_frame = self.replacer.insert_and_evict(page_id);
                if let Some(item) = evicted_frame {
                    self.free_list.push(item.id());
                    self.page_table.remove(&page_id);
                }
                None
            }
        }
    }
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
            Ok(val) => {}
            Err(err) => {
                assert_eq!(err.to_string(), "buffer will exceed page size");
            }
        }
    }

    #[test]
    fn test_read_write_page_frame() {
        let mut frame = Frame::new(5);
        let mut wp = WritePage {
            page_id: 1,
            pinned: Default::default(),
            frame: Arc::new(Mutex::new(&mut frame)),
        };

        let a = wp.write(&[97, 97]).unwrap();
        assert_eq!(a, 2);

        let mut rp = ReadPage {
            page_id: 1,
            pinned: Default::default(),
            frame: Arc::new(Mutex::new(&frame)),
        };

        let buf = rp.read();
        assert_eq!(buf, vec![97, 97, 0, 0, 0]);
    }
}
