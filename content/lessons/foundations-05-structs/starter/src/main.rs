struct User {
    first_name: String,
    last_name: String,
    email: String,
}

impl User {
    // (Faded) This method needs to read the struct's fields without moving them!
    // Add `&self` as the first parameter.
    fn full_name() -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    // (Open) Add `change_email` here!
    // It should take a mutable reference to self, and a new_email String.
}

fn main() {
    let mut user = User {
        first_name: String::from("Charles"),
        last_name: String::from("Russella"),
        email: String::from("charles@example.com"),
    };

    println!("User's full name is: {}", user.full_name());

    // (Open) Uncomment the lines below once you've written `change_email`:
    // user.change_email(String::from("charles.new@example.com"));
    // println!("Email updated to: {}", user.email);
}
