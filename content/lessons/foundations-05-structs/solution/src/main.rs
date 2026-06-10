struct User {
    first_name: String,
    last_name: String,
    email: String,
}

impl User {
    // (Faded)
    fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    // (Open)
    fn change_email(&mut self, new_email: String) {
        self.email = new_email;
    }
}

fn main() {
    let mut user = User {
        first_name: String::from("Charles"),
        last_name: String::from("Russella"),
        email: String::from("charles@example.com"),
    };

    println!("User's full name is: {}", user.full_name());

    // (Open)
    user.change_email(String::from("charles.new@example.com"));
    println!("Email updated to: {}", user.email);
}
