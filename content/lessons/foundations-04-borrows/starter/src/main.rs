fn print_length(s: &String) {
    println!("The length of '{}' is {}", s, s.len());
}

fn main() {
    // (Faded) The function `print_length` expects an immutable reference: `&String`.
    // But we are passing it by value, which moves it!
    // Fix this line by passing an immutable reference instead.
    let my_str = String::from("apple");
    print_length(my_str);
    println!("I still have my {}", my_str);

    // (Open) We want to print `r1` and then mutate using `r2`.
    // But `r2` creates a mutable reference while `r1` (immutable) is still in use!
    // Fix this by moving the `println!` that uses `r1` up, so `r1`'s usage ends before `r2` is created.
    let mut data = String::from("mutable");
    let r1 = &data;
    let r2 = &mut data;
    
    // We try to use r1 here!
    println!("r1 sees: {}", r1);
    
    r2.push_str(" and changed");
    println!("We mutated to: {}", r2);

    // (Faded) The compiler doesn't know which reference `longest` returns.
    // Fix the signature to use the `'a` lifetime: `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str`
    let word1 = String::from("apple");
    let word2 = String::from("banana");
    let longest_word = longest(&word1, &word2);
    println!("The longest word is: {}", longest_word);
}

fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
