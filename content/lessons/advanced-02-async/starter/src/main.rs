#![deny(unused_must_use)]

use std::time::Duration;
use tokio;

async fn say_hello() {
    println!("Hello from an async function!");
}

pub async fn step_1() {
    println!("Starting step 1...");
    // Calling the function returns a Future. 
    // We MUST `.await` it for the code inside it to actually run!
    say_hello().await;
}

pub async fn step_2() {
    println!("Starting step 2...");
    
    // TODO: We are trying to sleep for 50 milliseconds using `tokio::time::sleep`.
    // But we forgot to await the future! The compiler will warn us (and we've turned that warning into an error!).
    // Fix this code by appending `.await` to the sleep function.
    tokio::time::sleep(Duration::from_millis(50));
    
    println!("Finished step 2.");
}

pub async fn step_3() {
    let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![];
    
    for i in 0..3 {
        // TODO: Use `tokio::spawn(async move { ... })` to spawn a new task.
        // let handle = tokio::spawn(async move {
        //     println!("Task {} is running!", i);
        // });
        // handles.push(handle);
    }
    
    // Await all spawned tasks to finish
    for handle in handles {
        handle.await.unwrap();
    }
}

pub async fn fetch_url(url: &str) -> String {
    // Simulate a network request taking 100ms
    tokio::time::sleep(Duration::from_millis(100)).await;
    format!("Response from {}", url)
}

pub async fn step_4() -> Vec<String> {
    let urls = vec!["google.com", "rust-lang.org", "github.com"];
    
    // TODO: We want to fetch all 3 URLs concurrently. 
    // Option A: Use `tokio::spawn` to spawn a task for each URL, collect the handles, and await them all.
    // Option B: Use `tokio::join!(fetch_url(urls[0]), fetch_url(urls[1]), fetch_url(urls[2]))` to await them all concurrently.
    
    // Return a vector of the 3 responses!
    vec![]
}

#[tokio::main]
async fn main() {
    step_1().await;
    step_2().await;
    step_3().await;
    let responses = step_4().await;
    println!("Responses: {:?}", responses);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_step_2() {
        step_2().await;
    }

    #[tokio::test]
    async fn test_step_3() {
        step_3().await;
    }

    #[tokio::test]
    async fn test_step_4() {
        let res = step_4().await;
        assert_eq!(res.len(), 3);
        assert!(res.contains(&"Response from google.com".to_string()));
    }
}
