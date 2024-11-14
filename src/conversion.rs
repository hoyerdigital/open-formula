use crate::{
    eval::{eval_ref, Context},
    types::{Error, Value},
};

pub trait ConvertToScalar {
    fn convert_to_scalar(&self, ctx: &Context) -> Value;
}

impl ConvertToScalar for Value {
    fn convert_to_scalar(&self, _ctx: &Context) -> Value {
        todo!()
    }
}

pub trait ConvertToNumber {
    fn convert_to_number(&self, ctx: &Context) -> Value;
}

impl ConvertToNumber for Value {
    fn convert_to_number(&self, ctx: &Context) -> Value {
        match self {
            Value::Num(n) => Value::Num(*n),
            Value::Bool(b) => {
                if *b {
                    Value::Num(1f64)
                } else {
                    Value::Num(0f64)
                }
            }
            // TODO: Text to Number
            Value::Ref(r) => eval_ref(ctx, r),
            _ => Value::Err(Error::Value),
        }
    }
}
