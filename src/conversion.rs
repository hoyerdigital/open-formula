use crate::{
    eval::{eval_ref, Context},
    types::{Error, Value},
};

pub trait ConvertToScalar {
    fn convert_to_scalar(&self, ctx: &Context) -> Value;
}

impl ConvertToScalar for Value {
    fn convert_to_scalar(&self, ctx: &Context) -> Value {
        match self {
            Value::Num(_) | Value::Bool(_) | Value::String(_) => self.clone(),
            Value::Ref(r) => eval_ref(ctx, r),
            Value::Err(_) => self.clone(),
            _ => Value::Err(Error::Value),
        }
    }
}

pub trait ConvertToNumber {
    fn convert_to_number(&self, ctx: &Context) -> Value;
}

impl ConvertToNumber for Value {
    fn convert_to_number(&self, ctx: &Context) -> Value {
        match self {
            Value::Num(_) => self.clone(),
            Value::Bool(b) => {
                if *b {
                    Value::Num(1f64)
                } else {
                    Value::Num(0f64)
                }
            }
            Value::EmptyCell => Value::Num(0f64),
            // TODO: Text to Number
            Value::Ref(r) => eval_ref(ctx, r).convert_to_number(ctx),
            _ => Value::Err(Error::Value),
        }
    }
}
