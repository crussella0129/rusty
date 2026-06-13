#[derive(Debug, PartialEq)]
enum Expr {
    Literal(i32),
    // TODO: Wrap the recursive `Expr`s in a `Box` to give them a known size
    // Add(Expr, Expr),
}

// // TODO: Implement eval for Literal and Add
// fn eval(expr: &Expr) -> i32 {
//     0
// }

fn main() {
    // Step 2 validation
    /* Uncomment this block after completing Step 2
    let expr = Expr::Add(
        Box::new(Expr::Literal(5)),
        Box::new(Expr::Literal(10)),
    );
    println!("5 + 10 = {}", eval(&expr));
    */

    // Step 3 validation
    /* Uncomment this block after completing Step 3
    let expr2 = Expr::Multiply(
        Box::new(Expr::Add(
            Box::new(Expr::Literal(2)),
            Box::new(Expr::Literal(3)),
        )),
        Box::new(Expr::Literal(4)),
    );
    println!("(2 + 3) * 4 = {}", eval(&expr2));
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_add() {
        let e = Expr::Add(Box::new(Expr::Literal(2)), Box::new(Expr::Literal(3)));
        assert_eq!(eval(&e), 5);
    }
}
