use std::num::ParseIntError;

fn multiply_strings(s1: &str, s2: &str) -> Result<i32, ParseIntError> {
    // (Faded) We want to parse s1 and s2 to i32, propagating any parsing error
    // using the `?` operator. Fill in the missing `?` operators!
    let n1 = s1.parse::<i32>() /* TODO */;
    let n2 = s2.parse::<i32>() /* TODO */;
    Ok(n1 * n2)
}

// (Open) Implement `process_transaction` here!
// fn process_transaction(amount_str: &str, balance: i32) -> Result<i32, String> {
// ...
// }

fn main() {
    match multiply_strings("10", "5") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Failed to parse: {}", e),
    }
    match multiply_strings("10", "abc") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Failed to parse: {}", e),
    }

    // (Open) Uncomment the lines below once you've implemented `process_transaction`:
    /*
    match process_transaction("50", 100) {
        Ok(new_bal) => println!("New balance: {}", new_bal),
        Err(e) => println!("Transaction failed: {}", e),
    }
    match process_transaction("abc", 100) {
        Ok(new_bal) => println!("New balance: {}", new_bal),
        Err(e) => println!("Transaction failed: {}", e),
    }
    match process_transaction("-10", 100) {
        Ok(new_bal) => println!("New balance: {}", new_bal),
        Err(e) => println!("Transaction failed: {}", e),
    }
    match process_transaction("150", 100) {
        Ok(new_bal) => println!("New balance: {}", new_bal),
        Err(e) => println!("Transaction failed: {}", e),
    }
    */
}
