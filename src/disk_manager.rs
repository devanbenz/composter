use crate::DEFAULT_PAGE_SIZE;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub enum DiskManagerRequest {
    DiskRwRequest(DiskRequest),
    DiskIncreaseRequest(usize),
    DiskDecreaseRequest(usize),
}

pub struct DiskRequest {
    pub is_write: bool,
    pub data: Vec<u8>,
    pub page_id: usize,
    pub callback: Sender<bool>,
}

pub struct DiskManager {
    page_dir: Option<PathBuf>,
    file_handle: Option<File>,
    in_memory_pages: Option<Vec<u8>>,
    in_memory: bool,
    page_size: usize,
}

impl Default for DiskManager {
    fn default() -> Self {
        // Default page size is 4kb.
        let page_size = DEFAULT_PAGE_SIZE;
        Self::new(page_size, None, true)
    }
}

impl DiskManager {
    pub fn new(
        page_size: usize,
        page_dir: Option<std::path::PathBuf>,
        in_memory: bool,
    ) -> DiskManager {
        let mut in_memory_pages = None;
        let mut file_handle = None;
        if in_memory {
            in_memory_pages = Some(Vec::new());
        } else {
            let pd = page_dir.clone();
            in_memory_pages = None;
            let file = match Self::open_data_file(&pd.unwrap()) {
                Ok(f) => f,
                Err(err) => panic!("{}", err),
            };

            file_handle = Some(file);
        }

        DiskManager {
            page_dir,
            in_memory,
            in_memory_pages,
            page_size,
            file_handle,
        }
    }

    pub fn write_page(&mut self, request: DiskRequest) {
        let mut is_okay = true;
        if self.in_memory {
            match self.write_in_memory(request.page_id, request.data) {
                Ok(_) => {
                    is_okay = true;
                }
                Err(_) => {
                    is_okay = false;
                }
            }
        } else {
            match self.write_disk(request.page_id) {
                Ok(_) => {
                    is_okay = true;
                }
                Err(_) => {
                    is_okay = false;
                }
            }
        }
        request
            .callback
            .send(is_okay)
            .expect("failed to send to channel");
    }

    pub fn read_page(&self, request: DiskRequest) {
        let mut is_okay = true;
        if self.in_memory {
            match self.read_in_memory(request.page_id) {
                Ok(_) => {
                    is_okay = true;
                }
                Err(_) => {
                    is_okay = false;
                }
            }
        } else {
            match self.read_disk(request.page_id) {
                Ok(_) => {
                    is_okay = true;
                }
                Err(_) => {
                    is_okay = false;
                }
            }
        }
        request
            .callback
            .send(is_okay)
            .expect("failed to send to channel");
    }

    pub fn increase_pages(&mut self, p_id: usize) {
        if self.in_memory {
            let pages = self.in_memory_pages.as_mut().unwrap();
            pages.resize(p_id * self.page_size, 0);
        } else {
            let file = self.file_handle.as_mut().unwrap();
            file.set_len((p_id * self.page_size) as u64).unwrap();
        }
    }

    pub fn decrease_pages(&mut self, p_id: usize) {
        unimplemented!()
    }

    fn write_in_memory(&mut self, p_id: usize, p_data: Vec<u8>) -> Result<(), std::io::Error> {
        let pages = self.in_memory_pages.as_mut().unwrap();
        let offset = p_id * self.page_size;

        pages[offset..self.page_size].copy_from_slice(&p_data[offset..self.page_size]);
        Ok(())
    }

    fn write_disk(&self, p_id: usize) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn read_in_memory(&self, p_id: usize) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn read_disk(&self, p_id: usize) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn open_data_file(pd: &PathBuf) -> Result<File, std::io::Error> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(pd)
    }
}

mod tests {
    use super::*;
    #[test]
    fn test_disk_manager() {
        let mut dm = DiskManager::default();
        assert_eq!(dm.page_size, DEFAULT_PAGE_SIZE);
        assert_eq!(dm.page_dir, None);
        assert!(dm.in_memory);
        assert_eq!(dm.in_memory_pages, Some(vec![]));

        let v_test: Vec<u8> = vec![0; DEFAULT_PAGE_SIZE];

        dm.increase_pages(1);
        assert_eq!(dm.in_memory_pages, Some(v_test));

        let v_test: Vec<u8> = vec![u8::try_from('a').unwrap(); DEFAULT_PAGE_SIZE];
        dm.write_in_memory(1, v_test.clone()).unwrap();

        assert_eq!(dm.in_memory_pages, Some(v_test));
    }
}
