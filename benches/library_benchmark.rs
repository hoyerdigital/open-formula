use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use open_formula::eval::{eval, Cell, Context};
use open_formula::parser::{parser, Parser};
use open_formula::types::{Error, Expr, Ref, Result, Value};
use std::hint::black_box;

pub fn setup_context(current: &str, expr: &str) -> (Context, Expr) {
    let expr = parser()
        .parse(expr)
        .expect("expected valid formula expression");
    let current = parser()
        .parse(current)
        .expect("expected single cell position");
    if let Expr::Ref(Ref::CellRef(x, y)) = current {
        let mut ctx = Context {
            current_loc: Some((x, y)),
            ..Default::default()
        };

        // add at least one dynamic function, so that dynamic evaluation isn't
        // optimized away
        ctx.functions.insert(
            "UNIMPLEMENTED".into(),
            Box::new(|_, _| Err(Error::Unimplemented)),
        );

        ctx.sheet.set(
            0,
            0,
            Cell {
                value: Some(Value::Num(42.0)),
                expr: None,
            },
        );
        (ctx, expr)
    } else {
        panic!("expected single cell position");
    }
}

// TODO: add more complicated benchmark cases
#[library_benchmark(setup = setup_context)]
#[bench::simple_ref(args = ("B1", "A1"))]
#[bench::simple_trig(args = ("B1", "SIN(ABS(A1))"))]
fn bench_eval(args: (Context, Expr)) -> Result {
    let (ctx, expr) = args;
    black_box(eval(&ctx, &expr))
}

library_benchmark_group!(
    name = bench_eval_group;
    benchmarks = bench_eval
);

main!(library_benchmark_groups = bench_eval_group);
