fn print_countdown() {
    for number in (1..=3).rev() {
        println!("{}", number);
    }
    println!("Liftoff!");
}

fn main() {
    let mut counter = 0;

    let result = loop {
        counter += 1;

        if counter == 3 {
            break counter * 2;
        }
    };

    println!("The result is {}", result);

    print_countdown();
}
