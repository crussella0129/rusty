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

    // TODO: Print `my_string` inside the closure. 
    // You will notice the compiler complains because the thread might outlive `my_string`.
    // Add the `move` keyword before the closure `||` to force it to take ownership!
    let handle = thread::spawn(|| {
        
    });

    handle.join().unwrap();
}

pub fn step_3() -> String {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        let msg = String::from("Message from a thread");
        // TODO: Send `msg` through the transmitter (`tx`).
        // tx.send(...).unwrap();
    });

    // TODO: Receive the message from the receiver (`rx`) and return it.
    // let received = rx.recv().unwrap();
    // Return an empty string for now to make the starter project compile
    String::new()
}

pub fn step_4() -> i32 {
    // TODO: Create a shared counter wrapped in an Arc and a Mutex, initialized to 0.
    // let counter = Arc::new(Mutex::new(0));
    
    let mut handles = vec![];

    for _ in 0..3 {
        // TODO: Clone the Arc so this thread has its own reference
        // let counter_clone = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            // TODO: Lock the Mutex and increment the inner value by 1.
            // let mut num = counter_clone.lock().unwrap();
            // *num += 1;
        });
        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // TODO: Lock the Mutex one last time and return the final value!
    0
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
        // Just checking if it compiles and runs without panicking
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
