// fn print_countdown() {
//     /* TODO: Use a for loop from 1 to 3 reversed, and print each number. Then print "Liftoff!" */
// }

fn main() {
    let mut counter = 0;

    let result = loop {
        counter += 1;

        if counter == 3 {
            /* TODO: break returning counter * 2 */
            break 0; // Replace this
        }
    };

    // This checks Step 1
    println!("The result is {}", result);

    // Step 2 Validation:
    // print_countdown();
}
