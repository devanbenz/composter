use bincode::{serialize, serialized_size};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

type PageDirectory = HashMap<usize, usize>;

#[derive(Serialize, Deserialize)]
pub struct PageHeader {
    /// ID of the page
    page_id: usize,
    /// size of pages. defaults to 4kb
    page_size: usize,
}

impl PageHeader {
    pub fn new(page_size: usize, page_id: usize) -> PageHeader {
        PageHeader { page_size, page_id }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Page {
    /// head for pages on disk
    header: PageHeader,
    /// data for pages on disk
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct Pages {
    page_directory: PageDirectory,
    pages: Vec<Page>,
}

impl Pages {
    pub fn new_in_memory() -> Pages {
        Pages {
            page_directory: PageDirectory::new(),
            pages: Vec::new(),
        }
    }

    pub fn new_on_disk(path: PathBuf) -> File {
        if path.exists() && path.is_file() {
            let file = OpenOptions::new()
                .write(true)
                .read(true)
                .open(path)
                .unwrap();

            return file;
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .expect("Couldn't open page");

        let page_directory = PageDirectory::new();
        let mut pages = Vec::new();
        let pages = Pages {
            page_directory,
            pages,
        };

        let size = serialized_size(&pages).unwrap();
        println!("writing {} data to disk", size);
        let pages = serialize(&pages).unwrap();
        std::fs::write(path, pages).unwrap();

        file
    }
}
