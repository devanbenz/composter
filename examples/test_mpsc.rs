use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Create the channel
    let (tx, rx) = mpsc::channel();

    // Clone the sender for use in multiple threads if needed
    let tx_clone = tx.clone();

    // Spawn the dedicated receiver thread
    let receiver_handle = thread::spawn(move || {
        loop {
            // Use try_recv() for non-blocking receives
            match rx.try_recv() {
                Ok(message) => {
                    println!("Received: {}", message);
                    // Process your message here
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No message available, do other work or sleep briefly
                    thread::sleep(Duration::from_millis(100));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("Channel disconnected, shutting down receiver");
                    break;
                }
            }
        }
    });

    // Main thread can continue doing work
    for i in 0..5 {
        tx.send(i).unwrap();
        println!("Sent: {}", i);
    }

    // Example of sending from another thread
    thread::spawn(move || {
        for i in 5..10 {
            tx_clone.send(i).unwrap();
            println!("Sent from another thread: {}", i);
        }
    });

    // If you need to wait for the receiver to finish
    receiver_handle.join().unwrap();
}
