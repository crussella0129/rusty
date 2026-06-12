trait Operator {
    /* TODO: Define fn apply(&self, input: &str) -> String; */
}

struct ForwardDiff {
    prev_data: String,
}

impl Operator for ForwardDiff {
    /* TODO: Implement apply. 
       Hint: split `input` by '~'. If an item is not in `self.prev_data`, it's new.
       Return the new items formatted as "New: item". If multiple, join with '~'.
    */
}

// TODO: For step 2, uncomment and implement the generic `process` function:
// fn process<T: Operator>(op: T, input: &str) -> String {
//     /* call apply on op */
// }

fn main() {
    let diff = ForwardDiff {
        prev_data: String::from("Apple~Banana"),
    };
    
    // This will not compile until Operator is implemented for ForwardDiff!
    // println!("{}", diff.apply("Apple~Banana~Orange"));

    // Step 2 Validation:
    // let diff2 = ForwardDiff { prev_data: String::from("Apple~Banana") };
    // let result = process(diff2, "Apple~Banana~Orange");
    // println!("Processed: {}", result);
}
