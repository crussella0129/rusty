pub fn greeting(name: &str) -> String {
    format!("Hello, {}", name)
}

pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeting() {
        assert_eq!(greeting("Rusty"), "Hello, Rusty");
    }

    #[test]
    fn test_is_even() {
        assert!(is_even(4));
        assert!(!is_even(5));
    }
}
