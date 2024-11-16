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

pub fn abs(args: &[Value], ctx: &Context) -> Result<Value, Error> {
    single_num_fn(args, ctx, |x| Ok(x.abs()))
}
