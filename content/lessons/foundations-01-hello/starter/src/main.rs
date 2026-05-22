fn main() {
    // (Open exercise) Change this line so `cargo run` prints exactly:
    //     I compiled my first Rust program!
    println!("Welcome — Rusty is compiling your first program.");
}

// (Faded exercise) The test below calls `greeting()`, which doesn't exist yet.
// Run `cargo test`, read the error Rusty shows you, then define the function here so
// it returns the String "Hello, Rusty!" — for example:
//
//     fn greeting() -> String {
//         String::from("Hello, Rusty!")
//     }

#[cfg(test)]
mod tests {
    #[test]
    fn greets_rusty() {
        assert_eq!(super::greeting(), "Hello, Rusty!");
    }
}
