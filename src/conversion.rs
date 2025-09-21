use crate::{
    eval::{eval_ref, Context},
    types::{Error, Result, Value},
};
use std::str::FromStr;

pub trait ConvertToScalar {
    fn convert_to_scalar(self, ctx: &Context) -> Result<Value>;
}

impl ConvertToScalar for Result<Value> {
    fn convert_to_scalar(self, ctx: &Context) -> Result<Value> {
        match self {
            Ok(Value::Num(_)) | Ok(Value::Bool(_)) | Ok(Value::String(_)) => self,
            Ok(Value::Ref(r)) => eval_ref(ctx, &r),
            Err(_) => self,
            _ => Err(Error::Value),
        }
    }
}

pub trait ConvertToNumber {
    fn convert_to_number(self, ctx: &Context) -> Result<Value>;
}

impl ConvertToNumber for Result<Value> {
    fn convert_to_number(self, ctx: &Context) -> Result<Value> {
        match self {
            Ok(Value::Num(_)) => self,
            Ok(Value::Bool(b)) => {
                if b {
                    Ok(Value::Num(1f64))
                } else {
                    Ok(Value::Num(0f64))
                }
            }
            Ok(Value::EmptyCell) => Ok(Value::Num(0f64)),
            Ok(Value::String(s)) => f64::from_str(&s).map(Value::Num).map_err(|_| Error::Value),
            Ok(Value::Ref(r)) => eval_ref(ctx, &r).convert_to_number(ctx),
            Err(_) => self,
        }
    }
}

pub trait ConvertToLogical {
    fn convert_to_logical(self, ctx: &Context) -> Result<Value>;
}

impl ConvertToLogical for Result<Value> {
    fn convert_to_logical(self, ctx: &Context) -> Result<Value> {
        match self {
            Ok(Value::Num(n)) => Ok(Value::Bool(n != 0.0)),
            // TODO: it may be possible to parse text to bool
            Ok(Value::String(_)) => Ok(Value::Bool(false)),
            Ok(Value::Bool(_)) => self,
            Ok(Value::Ref(_)) => self.convert_to_scalar(ctx).convert_to_logical(ctx),
            Ok(Value::EmptyCell) => Ok(Value::Bool(false)),
            Err(_) => self,
        }
    }
}

pub trait ConvertToText {
    fn convert_to_text(self, ctx: &Context) -> Result<Value>;
}

impl ConvertToText for Result<Value> {
    fn convert_to_text(self, ctx: &Context) -> Result<Value> {
        match self {
            Ok(Value::Num(n)) => {
                let mut buffer = ryu::Buffer::new();
                let fmt = buffer.format(n);
                Ok(Value::String(fmt.into()))
            }
            Ok(Value::String(_)) => self,
            Ok(Value::Bool(b)) => {
                if b {
                    Ok(Value::String("TRUE".to_string()))
                } else {
                    Ok(Value::String("FALSE".to_string()))
                }
            }
            Ok(Value::Ref(_)) => self.convert_to_scalar(ctx).convert_to_text(ctx),
            Ok(Value::EmptyCell) => Ok(Value::String("".to_string())),
            Err(_) => self,
        }
    }
}
