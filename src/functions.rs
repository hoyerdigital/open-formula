use crate::conversion::*;
use crate::eval::{eval, Context};
use crate::types::{Error, Expr, Result, Value};

fn single_num_fn<F>(args: &[Expr], ctx: &Context, f: F) -> Result<Value>
where
    F: Fn(f64) -> Result<f64>,
{
    if args.len() != 1 {
        Err(Error::Args)
    } else if let Value::Num(n) = eval(ctx, args.first().unwrap()).convert_to_number(ctx)? {
        Ok(Value::Num(f(n)?))
    } else {
        unreachable!()
    }
}

fn single_num_constraint_fn<C, F>(
    args: &[Expr],
    ctx: &Context,
    constraint: C,
    f: F,
) -> Result<Value>
where
    C: Fn(f64) -> bool,
    F: Fn(f64) -> Result<f64>,
{
    single_num_fn(args, ctx, |x| {
        if constraint(x) {
            Ok(f(x)?)
        } else {
            Err(Error::Num)
        }
    })
}

fn single_num_range_fn<F, R>(args: &[Expr], ctx: &Context, range: R, f: F) -> Result<Value>
where
    F: Fn(f64) -> Result<f64>,
    R: std::ops::RangeBounds<f64>,
{
    single_num_constraint_fn(args, ctx, |x| range.contains(&x), f)
}

pub fn abs(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_fn(args, ctx, |x| Ok(x.abs()))
}

pub fn acos(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_range_fn(args, ctx, -1.0..=1.0, |x| Ok(x.acos()))
}

pub fn asin(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_range_fn(args, ctx, -1.0..=1.0, |x| Ok(x.asin()))
}

pub fn atan(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_fn(args, ctx, |x| Ok(x.atan()))
}

pub fn cos(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_fn(args, ctx, |x| Ok(x.cos()))
}

pub fn degrees(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_fn(args, ctx, |x| Ok(x.to_degrees()))
}

pub fn exp(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_fn(args, ctx, |x| Ok(x.exp()))
}

pub fn ln(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_constraint_fn(args, ctx, |x| x > 0.0, |x| Ok(x.ln()))
}

pub fn log10(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_constraint_fn(args, ctx, |x| x > 0.0, |x| Ok(x.log10()))
}

pub fn radians(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_constraint_fn(args, ctx, |x| x > 0.0, |x| Ok(x.to_radians()))
}

pub fn sin(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_fn(args, ctx, |x| Ok(x.sin()))
}

pub fn sqrt(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_constraint_fn(args, ctx, |x| x >= 0.0, |x| Ok(x.sqrt()))
}

pub fn tan(args: &[Expr], ctx: &Context) -> Result<Value> {
    single_num_fn(args, ctx, |x| Ok(x.tan()))
}
