use open_formula::prelude::*;

fn main() {
    let mut context = Context::default();
    // Add two cells to our worksheet:
    // A   | B
    // ---------
    // 5.0 | 2.0
    context.sheet.set(
        0,
        0,
        Cell {
            value: Some(Value::Num(5.0)),
            expr: None,
        },
    );
    context.sheet.set(
        1,
        0,
        Cell {
            value: Some(Value::Num(2.0)),
            expr: None,
        },
    );
    // Parse formula string
    let expr = parser().parse("A1+B1").unwrap();
    // Evaluate formula expression
    println!("{:?}", eval(&context, &expr));
    // Ok(Num(7.0))
}
