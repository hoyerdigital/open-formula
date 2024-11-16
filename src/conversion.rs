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
            Value::Err(_) => self.clone(),
            _ => Value::Err(Error::Value),
        }
    }
}

pub trait ConvertToLogical {
    fn convert_to_logical(&self, ctx: &Context) -> Value;
}

impl ConvertToLogical for Value {
    fn convert_to_logical(&self, ctx: &Context) -> Value {
        match self {
            Value::Num(n) => Value::Bool(*n != 0.0),
            // TODO: it may be possible to parse text to bool
            Value::String(_) => Value::Bool(false),
            Value::Bool(_) => self.clone(),
            Value::Ref(_) => self.convert_to_scalar(ctx).convert_to_logical(ctx),
            Value::EmptyCell => Value::Bool(false),
            Value::Err(_) => self.clone(),
        }
    }
}

pub trait ConvertToText {
    fn convert_to_text(&self, ctx: &Context) -> Value;
}

impl ConvertToText for Value {
    fn convert_to_text(&self, ctx: &Context) -> Value {
        match self {
            Value::Num(n) => {
                let mut buffer = ryu::Buffer::new();
                let fmt = buffer.format(*n);
                Value::String(fmt.into())
            }
            Value::String(_) => self.clone(),
            Value::Bool(b) => {
                if *b {
                    Value::String("TRUE".to_string())
                } else {
                    Value::String("FALSE".to_string())
                }
            }
            Value::Ref(_) => self.convert_to_scalar(ctx).convert_to_text(ctx),
            Value::EmptyCell => Value::String("".to_string()),
            Value::Err(_) => self.clone(),
        }
    }
}
