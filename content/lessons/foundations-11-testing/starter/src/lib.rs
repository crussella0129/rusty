pub fn greeting(name: &str) -> String {
    format!("Hello, {}", name)
}

// TODO: Step 2: Add `pub fn is_even(n: i32) -> bool`

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add the `#[test]` attribute here
    fn test_greeting() {
        // TODO: fill in the assert_eq! macro
        // assert_eq!(/* your code */);
    }

    // TODO: Step 2: Write `test_is_even` here
}
