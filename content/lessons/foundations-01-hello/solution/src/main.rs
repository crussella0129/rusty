fn main() {
    println!("I compiled my first Rust program!");
}

fn greeting() -> String {
    String::from("Hello, Rusty!")
}

#[cfg(test)]
mod tests {
    #[test]
    fn greets_rusty() {
        assert_eq!(super::greeting(), "Hello, Rusty!");
    }
}
