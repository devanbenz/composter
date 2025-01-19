use std::io::{BufReader, Read, Seek, Write};

fn main() {
    let mut buf = [0; 1024];
    let mut buf2 = [0; 1024 * 2];
    let write_data = ['a'; 10];
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open("/Users/devan/Documents/Projects/composter/scratch_page/test_write2")
        .unwrap();
    file.set_len(1024).unwrap();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    file.write_all(b"data").unwrap();
    file.flush().unwrap();

    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    file.read_exact(buf.as_mut_slice()).unwrap();
    println!("{:?}", buf);

    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    file.set_len(1024 * 2).unwrap();
    println!("\n\n\n\n\n");
    file.read_exact(buf2.as_mut_slice()).unwrap();
    println!("{:?}", buf2);
}
