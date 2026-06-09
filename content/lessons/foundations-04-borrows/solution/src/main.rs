fn print_length(s: &String) {
    println!("The length of '{}' is {}", s, s.len());
}

fn main() {
    // (Faded)
    let my_str = String::from("apple");
    print_length(&my_str);
    println!("I still have my {}", my_str);

    // (Open)
    let mut data = String::from("mutable");
    let r1 = &data;
    println!("r1 sees: {}", r1); // Moved up to end the borrow here
    
    let r2 = &mut data;
    r2.push_str(" and changed");
    println!("We mutated to: {}", r2);

    // (Faded)
    let word1 = String::from("apple");
    let word2 = String::from("banana");
    let longest_word = longest(&word1, &word2);
    println!("The longest word is: {}", longest_word);
}

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
