use std::io::{Seek, Write};

fn main() {
    let m_vec = vec!['\0'; 1024];
    let bytesofdata = vec!['a'; 100];
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open("/Users/devan/Documents/Projects/composter/scratch_page/test_write")
        .unwrap();

    file.set_len(m_vec.len() as u64).unwrap();
    file.seek(std::io::SeekFrom::Start(16)).unwrap();

    let serialized = bincode::serialize(&bytesofdata).unwrap();
    file.write_all(serialized.as_slice()).unwrap();
    file.flush().unwrap();
}
