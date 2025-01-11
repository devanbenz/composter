pub struct DiskManager {}

pub struct DiskRequest {
    pub is_write: bool,
    pub data: Vec<u8>,
    pub page_id: usize,
    pub callback: Option<Box<dyn FnMut(String)>>,
}

impl DiskManager {
    pub fn write_page(&self, _request: DiskRequest) {
        println!("received write page");
    }

    pub fn read_page(&self, _request: DiskRequest) {
        println!("received read page");
    }
}
