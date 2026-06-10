fn print_nickname(nickname: Option<String>) {
    // (Faded)
    match nickname {
        Some(name) => println!("My nickname is: {}", name),
        None => println!("No nickname"),
    }
}

// (Open)
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Echo(String),
}

fn process_message(msg: Message) {
    // (Open)
    match msg {
        Message::Quit => println!("Quitting..."),
        Message::Move { x, y } => println!("Moving to x: {}, y: {}", x, y),
        Message::Echo(text) => println!("Echo: {}", text),
    }
}

fn main() {
    let has_nick = Some(String::from("Rusty"));
    let no_nick: Option<String> = None;
    
    print_nickname(has_nick);
    print_nickname(no_nick);

    let msg1 = Message::Quit;
    let msg2 = Message::Move { x: 10, y: 20 };
    
    // (Open)
    let msg3 = Message::Echo(String::from("Hello!"));
    
    process_message(msg1);
    process_message(msg2);
    process_message(msg3);
}
