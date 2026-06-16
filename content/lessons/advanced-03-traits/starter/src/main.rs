use std::fmt::Display;

pub fn print_it<T>(item: T) where T: Display {
    println!("Printing a generic item: {}", item);
}

pub fn step_1() {
    print_it(42);         
    print_it("Hello!");   
}

pub trait Graph {
    type Node;
    fn edges(&self, node: &Self::Node) -> Vec<Self::Node>;
}

pub struct MyGraph;

// TODO: The compiler complains because we haven't specified the associated type `Node`!
// Inside this impl block, add `type Node = i32;`
impl Graph for MyGraph {
    fn edges(&self, _node: &Self::Node) -> Vec<Self::Node> {
        vec![]
    }
}

pub fn step_2() {
    let graph = MyGraph;
    let _edges = graph.edges(&0);
}

pub struct CustomPointer {
    pub data: String,
}

// TODO: Implement the `Drop` trait for `CustomPointer`.
// Inside the `drop` method, print "Dropping CustomPointer!" to the console.


pub fn step_3() {
    let _ptr = CustomPointer { data: String::from("secret data") };
    // _ptr will be dropped at the end of this scope!
}

// TODO: For step 4, create a generic function `print_default<T>()`. 
// Use a `where` clause to require that `T` implements both `Default` and `Display`. 
// Inside the function, instantiate the default value using `T::default()` and print it.


pub fn step_4() {
    // print_default::<i32>();
    // print_default::<String>();
}

fn main() {
    step_1();
    step_2();
    step_3();
    step_4();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_2() {
        step_2();
    }

    #[test]
    fn test_step_3() {
        // TODO: Implement Drop for CustomPointer, then delete this panic!
        panic!("Implement Drop!");
    }

    #[test]
    fn test_step_4() {
        // TODO: Call your `print_default` function here with `<i32>`
        panic!("Implement print_default and update this test!");
    }
}
