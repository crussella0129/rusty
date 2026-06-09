fn main() {
    // (Faded) Make `x` mutable so we can change it to 6.
    let mut x = 5;
    x = 6;
    println!("x is {}", x);

    // (Open)
    let spaces = "   ";
    let spaces = spaces.len();
    println!("There are {} spaces.", spaces);
}
