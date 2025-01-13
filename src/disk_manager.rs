use crate::page::{Page, Pages};
use std::fs::File;
use std::io::{stdout, Error};
use std::sync::mpsc::Sender;

pub struct DiskRequest {
    pub is_write: bool,
    pub data: Vec<u8>,
    pub page_id: usize,
    pub callback: Sender<bool>,
}

pub struct DiskManager {
    page_dir: Option<std::path::PathBuf>,
    in_memory_pages: Option<Pages>,
    file_handler: Option<File>,
    in_memory: bool,
    page_size: usize,
}

impl Default for DiskManager {
    fn default() -> Self {
        // Default page size is 4kb.
        let page_size = 4 * 1024;
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
        let mut file_handler = None;
        if in_memory {
            in_memory_pages = Some(Pages::new_in_memory());
        } else {
            let pd = page_dir.clone();
            file_handler = Some(Pages::new_on_disk(pd.unwrap()));
        }

        DiskManager {
            page_dir,
            in_memory,
            in_memory_pages,
            page_size,
            file_handler,
        }
    }

    pub fn write_page(&self, request: DiskRequest) {
        let mut is_okay = true;
        if self.in_memory {
            match self.write_in_memory() {
                Ok(_) => {}
                Err(_) => {
                    is_okay = false;
                }
            }
        } else {
            match self.write_disk() {
                Ok(_) => {}
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
            match self.read_in_memory() {
                Ok(_) => {}
                Err(_) => {
                    is_okay = false;
                }
            }
        } else {
            match self.read_disk() {
                Ok(_) => {}
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

    fn write_in_memory(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn write_disk(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn read_in_memory(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn read_disk(&self) -> Result<(), std::io::Error> {
        Ok(())
    }
}
