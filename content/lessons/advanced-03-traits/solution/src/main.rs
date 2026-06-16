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

impl Graph for MyGraph {
    type Node = i32;
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

impl Drop for CustomPointer {
    fn drop(&mut self) {
        println!("Dropping CustomPointer!");
    }
}

pub fn step_3() {
    let _ptr = CustomPointer { data: String::from("secret data") };
    // _ptr will be dropped at the end of this scope!
}

pub fn print_default<T>() where T: Default + Display {
    let val = T::default();
    println!("{}", val);
}

pub fn step_4() {
    print_default::<i32>();
    print_default::<String>();
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
        step_3();
    }

    #[test]
    fn test_step_4() {
        print_default::<i32>();
    }
}
