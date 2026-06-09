fn eat_string(s: String) {
    println!("Ate the string: {}", s);
}

fn main() {
    // (Faded)
    let s1 = String::from("hello");
    let s2 = s1.clone();
    println!("s1 is {} and s2 is {}", s1, s2);

    // (Open)
    let my_str = String::from("yummy");
    eat_string(my_str.clone());
    println!("Still have my string: {}", my_str);
}
