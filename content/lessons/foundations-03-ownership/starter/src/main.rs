fn eat_string(s: String) {
    println!("Ate the string: {}", s);
}

fn main() {
    // (Faded) s1 is moved into s2, which makes s1 invalid.
    // Fix this error by using `.clone()` to deep copy s1 instead.
    let s1 = String::from("hello");
    let s2 = s1;
    println!("s1 is {} and s2 is {}", s1, s2);

    // (Open) We want to call `eat_string` but still use `my_str` afterwards.
    // However, `eat_string` takes ownership of the string!
    // Fix the call to `eat_string` so this compiles.
    let my_str = String::from("yummy");
    eat_string(my_str);
    println!("Still have my string: {}", my_str);
}
