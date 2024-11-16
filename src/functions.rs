use crate::conversion::*;
use crate::eval::Context;
use crate::types::Error;
use crate::types::Value;

fn single_num_fn<F>(args: &[Value], ctx: &Context, f: F) -> Result<Value, Error>
where
    F: Fn(f64) -> Result<f64, Error>,
{
    if args.len() != 1 {
        Err(Error::Args)
    } else if let Value::Num(n) = args.first().unwrap().convert_to_number(ctx).into_result()? {
        Ok(Value::Num(f(n)?))
    } else {
        unreachable!()
    }
}

fn single_num_constraint_fn<C, F>(
    args: &[Value],
    ctx: &Context,
    constraint: C,
    f: F,
) -> Result<Value, Error>
where
    C: Fn(f64) -> bool,
    F: Fn(f64) -> Result<f64, Error>,
{
    single_num_fn(args, ctx, |x| {
        if constraint(x) {
            Ok(f(x)?)
        } else {
            Err(Error::Num)
        }
    })
}

fn single_num_range_fn<F, R>(args: &[Value], ctx: &Context, range: R, f: F) -> Result<Value, Error>
where
    F: Fn(f64) -> Result<f64, Error>,
    R: std::ops::RangeBounds<f64>,
{
    single_num_constraint_fn(args, ctx, |x| range.contains(&x), f)
}

pub fn abs(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_fn(args, ctx, |x| Ok(x.abs()))
}

pub fn acos(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_range_fn(args, ctx, -1.0..=1.0, |x| Ok(x.acos()))
}

pub fn asin(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_range_fn(args, ctx, -1.0..=1.0, |x| Ok(x.asin()))
}

pub fn atan(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_fn(args, ctx, |x| Ok(x.atan()))
}

pub fn cos(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_fn(args, ctx, |x| Ok(x.cos()))
}

pub fn degrees(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_fn(args, ctx, |x| Ok(x.to_degrees()))
}

pub fn exp(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_fn(args, ctx, |x| Ok(x.exp()))
}

pub fn ln(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_constraint_fn(args, ctx, |x| x > 0.0, |x| Ok(x.ln()))
}

pub fn log10(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_constraint_fn(args, ctx, |x| x > 0.0, |x| Ok(x.log10()))
}

pub fn radians(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_constraint_fn(args, ctx, |x| x > 0.0, |x| Ok(x.to_radians()))
}

pub fn sin(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_fn(args, ctx, |x| Ok(x.sin()))
}

pub fn sqrt(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_constraint_fn(args, ctx, |x| x >= 0.0, |x| Ok(x.sqrt()))
}

pub fn tan(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_fn(args, ctx, |x| Ok(x.tan()))
}
