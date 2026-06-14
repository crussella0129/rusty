use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn step_1() {
    let handle = thread::spawn(|| {
        println!("Hello from the spawned thread!");
        thread::sleep(Duration::from_millis(50));
    });

    println!("Hello from the main thread!");

    // Wait for the spawned thread to finish
    handle.join().unwrap();
}

pub fn step_2() {
    let my_string = String::from("Fearless Concurrency");

    let handle = thread::spawn(move || {
        println!("The thread says: {}", my_string);
    });

    handle.join().unwrap();
}

pub fn step_3() -> String {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        let msg = String::from("Message from a thread");
        tx.send(msg).unwrap();
    });

    rx.recv().unwrap()
}

pub fn step_4() -> i32 {
    let counter = Arc::new(Mutex::new(0));
    
    let mut handles = vec![];

    for _ in 0..3 {
        let counter_clone = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    let final_val = *counter.lock().unwrap();
    final_val
}

fn main() {
    println!("Step 1:");
    step_1();
    
    println!("\nStep 2:");
    step_2();
    
    println!("\nStep 3:");
    println!("Received: {}", step_3());
    
    println!("\nStep 4:");
    println!("Final counter value: {}", step_4());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_2() {
        step_2();
    }

    #[test]
    fn test_step_3() {
        assert_eq!(step_3(), "Message from a thread");
    }

    #[test]
    fn test_step_4() {
        assert_eq!(step_4(), 3);
    }
}
