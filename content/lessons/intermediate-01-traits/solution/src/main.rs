trait Operator {
    fn apply(&self, input: &str) -> String;
}

struct ForwardDiff {
    prev_data: String,
}

impl Operator for ForwardDiff {
    fn apply(&self, input: &str) -> String {
        let mut new_items = Vec::new();
        for item in input.split('~') {
            if !self.prev_data.contains(item) {
                new_items.push(format!("New: {}", item));
            }
        }
        new_items.join("~")
    }
}

fn process<T: Operator>(op: T, input: &str) -> String {
    op.apply(input)
}

fn main() {
    let diff = ForwardDiff {
        prev_data: String::from("Apple~Banana"),
    };
    
    println!("{}", diff.apply("Apple~Banana~Orange"));

    let diff2 = ForwardDiff { prev_data: String::from("Apple~Banana") };
    let result = process(diff2, "Apple~Banana~Orange");
    println!("Processed: {}", result);
}
