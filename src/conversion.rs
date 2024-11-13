use crate::types::{Error, Value};

pub trait ConvertToScalar {
    fn convert_to_scalar(&self) -> Value;
}

impl ConvertToScalar for Value {
    fn convert_to_scalar(&self) -> Value {
        todo!()
    }
}

pub trait ConvertToNumber {
    fn convert_to_number(&self) -> Value;
}

impl ConvertToNumber for Value {
    fn convert_to_number(&self) -> Value {
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
            // TODO: Reference to Number
            _ => Value::Err(Error::Value),
        }
    }
}
