fn print_nickname(nickname: Option<String>) {
    // (Faded) This match statement is missing the `None` arm!
    // Add it so it prints "No nickname".
    match nickname {
        Some(name) => println!("My nickname is: {}", name),
    }
}

// (Open) Add a new variant `Echo(String)` to this enum.
enum Message {
    Quit,
    Move { x: i32, y: i32 },
}

fn process_message(msg: Message) {
    // (Open) Once you add the Echo variant above, this match will fail to compile
    // because it isn't exhaustive! Add an arm for `Message::Echo(text)` that prints "Echo: {}"
    match msg {
        Message::Quit => println!("Quitting..."),
        Message::Move { x, y } => println!("Moving to x: {}, y: {}", x, y),
    }
}

fn main() {
    let has_nick = Some(String::from("Rusty"));
    let no_nick: Option<String> = None;
    
    print_nickname(has_nick);
    print_nickname(no_nick);

    let msg1 = Message::Quit;
    let msg2 = Message::Move { x: 10, y: 20 };
    
    // (Open) Create msg3 here with Message::Echo(String::from("Hello!"))
    
    process_message(msg1);
    process_message(msg2);
    // process_message(msg3); // Uncomment this line once msg3 is created
}
