use std::num::ParseIntError;

fn multiply_strings(s1: &str, s2: &str) -> Result<i32, ParseIntError> {
    // (Faded)
    let n1 = s1.parse::<i32>()?;
    let n2 = s2.parse::<i32>()?;
    Ok(n1 * n2)
}

// (Open)
fn process_transaction(amount_str: &str, balance: i32) -> Result<i32, String> {
    let amount = match amount_str.parse::<i32>() {
        Ok(val) => val,
        Err(_) => return Err(String::from("Invalid amount format")),
    };
    if amount <= 0 {
        return Err(String::from("Amount must be positive"));
    }
    if amount > balance {
        return Err(String::from("Insufficient funds"));
    }
    Ok(balance - amount)
}

fn main() {
    match multiply_strings("10", "5") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Failed to parse: {}", e),
    }
    match multiply_strings("10", "abc") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Failed to parse: {}", e),
    }

    // (Open)
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
}
