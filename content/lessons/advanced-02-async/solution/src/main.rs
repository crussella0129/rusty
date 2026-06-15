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
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    println!("Finished step 2.");
}

pub async fn step_3() {
    let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![];
    
    for i in 0..3 {
        let handle = tokio::spawn(async move {
            println!("Task {} is running!", i);
        });
        handles.push(handle);
    }
    
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
    
    let (r1, r2, r3) = tokio::join!(
        fetch_url(urls[0]),
        fetch_url(urls[1]),
        fetch_url(urls[2])
    );
    
    vec![r1, r2, r3]
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
