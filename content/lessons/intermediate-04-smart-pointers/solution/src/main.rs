#[derive(Debug, PartialEq)]
enum Expr {
    Literal(i32),
    Add(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
}

fn eval(expr: &Expr) -> i32 {
    match expr {
        Expr::Literal(val) => *val,
        Expr::Add(left, right) => eval(left) + eval(right),
        Expr::Multiply(left, right) => eval(left) * eval(right),
    }
}

fn main() {
    // Step 2 validation
    let expr = Expr::Add(
        Box::new(Expr::Literal(5)),
        Box::new(Expr::Literal(10)),
    );
    println!("5 + 10 = {}", eval(&expr));

    // Step 3 validation
    let expr2 = Expr::Multiply(
        Box::new(Expr::Add(
            Box::new(Expr::Literal(2)),
            Box::new(Expr::Literal(3)),
        )),
        Box::new(Expr::Literal(4)),
    );
    println!("(2 + 3) * 4 = {}", eval(&expr2));
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
