use composter::buffer_pool_manager::{Frame, WritePage};
use std::io::Write;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

fn main() {
    let v1: Vec<u8> = vec![97, 97, 0, 0, 0];
    let v2: Vec<u8> = vec![97, 97, 97, 0, 0];
    let v3: Vec<u8> = vec![97, 97, 97, 97, 0];
    let v1_t = v1.iter().take_while(|x| *x != &0).collect::<Vec<_>>();
    println!("{} {}", v1.len(), v1_t.len());
}
