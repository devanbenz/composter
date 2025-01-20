use crate::DEFAULT_PAGE_SIZE;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub enum DiskManagerRequest {
    DiskRwRequest {
        is_write: bool,
        data: Arc<Mutex<Vec<u8>>>,
        page_id: usize,
        callback: Sender<bool>,
    },
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

    pub fn write_page(&mut self, data: &mut Vec<u8>, page_id: usize, callback: Sender<bool>) {
        let mut is_okay = true;
        if self.in_memory {
            match self.write_in_memory(page_id, data) {
                Ok(_) => {
                    is_okay = true;
                }
                Err(_) => {
                    is_okay = false;
                }
            }
        } else {
            match self.write_disk(page_id) {
                Ok(_) => {
                    is_okay = true;
                }
                Err(_) => {
                    is_okay = false;
                }
            }
        }

        callback.send(is_okay).expect("failed to send to channel");
    }

    pub fn read_page(&mut self, data: &mut Vec<u8>, page_id: usize, callback: Sender<bool>) {
        let mut is_okay = true;
        if self.in_memory {
            match self.read_in_memory(page_id, data) {
                Ok(_) => {
                    is_okay = true;
                }
                Err(_) => {
                    is_okay = false;
                }
            }
        } else {
            match self.read_disk(page_id, data) {
                Ok(_) => {
                    is_okay = true;
                }
                Err(_) => {
                    is_okay = false;
                }
            }
        }

        callback.send(is_okay).expect("failed to send to channel");
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

    fn write_in_memory(&mut self, p_id: usize, p_data: &mut Vec<u8>) -> Result<(), std::io::Error> {
        let pages = match &mut self.in_memory_pages {
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "no pages available",
                ));
            }
            Some(pages) => pages,
        };

        let offset = (p_id - 1) * self.page_size;
        pages[offset..self.page_size].copy_from_slice(&p_data);
        Ok(())
    }

    fn write_disk(&self, p_id: usize) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn read_in_memory(&mut self, p_id: usize, p_data: &mut Vec<u8>) -> Result<(), std::io::Error> {
        let pages = match &mut self.in_memory_pages {
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "no pages available",
                ));
            }
            Some(pages) => pages,
        };

        let offset = (p_id - 1) * self.page_size;
        p_data.copy_from_slice(&pages[offset..self.page_size]);
        Ok(())
    }

    fn read_disk(&self, p_id: usize, p_data: &mut Vec<u8>) -> Result<(), std::io::Error> {
        let mut buf_reader = BufReader::new(self.file_handle.as_ref().unwrap());
        buf_reader.seek(SeekFrom::Start(((p_id - 1) * self.page_size) as u64))?;
        buf_reader.read_exact(p_data)?;

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
    use tempdir::TempDir;
    #[test]
    fn test_disk_manager_in_memory() {
        let mut dm = DiskManager::default();
        assert_eq!(dm.page_size, DEFAULT_PAGE_SIZE);
        assert_eq!(dm.page_dir, None);
        assert!(dm.in_memory);
        assert_eq!(dm.in_memory_pages, Some(vec![]));

        let v_test: Vec<u8> = vec![0; DEFAULT_PAGE_SIZE];

        dm.increase_pages(1);
        assert_eq!(dm.in_memory_pages, Some(v_test));

        let mut v_test: Vec<u8> = vec![u8::try_from('a').unwrap(); DEFAULT_PAGE_SIZE];
        dm.write_in_memory(1, &mut v_test).unwrap();
        assert_eq!(dm.in_memory_pages, Some(v_test));

        let mut v_test: Vec<u8> = vec![0; DEFAULT_PAGE_SIZE];
        dm.read_in_memory(1, &mut v_test).unwrap();
        assert_eq!(dm.in_memory_pages, Some(v_test));
    }

    #[test]
    fn test_disk_manager_on_disk() {
        let temp_dir = TempDir::new("test_disk_manager").unwrap();
        let temp_file = temp_dir.path().join("test.db");
        let (tx, rx) = std::sync::mpsc::channel();

        let mut dm = DiskManager::new(DEFAULT_PAGE_SIZE, Some(temp_file), false);
        assert_eq!(dm.page_size, DEFAULT_PAGE_SIZE);
        assert!(!dm.in_memory);

        // Blank page
        let mut v_test: Vec<u8> = vec![0; DEFAULT_PAGE_SIZE];
        dm.increase_pages(1);
        dm.read_page(&mut v_test, 1, tx);

        let recv = rx.recv().unwrap();
        assert!(recv);

        drop(dm);
        temp_dir.close().unwrap();
    }
}
