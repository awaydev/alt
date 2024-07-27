use super::{r#gen::popv, value::Value};
use std::fmt;

impl fmt::Display for Value {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Number(r) => write!(f, "{r}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Array(x) => write!(f, "{x:?}"),
            Value::Block(x) => write!(f, "{}", popv(&mut x.clone()).unwrap()),
            Value::Ref(x) => write!(f, "{}", x.clone()),
            _ => write!(f, "{self:?}")
        }
    }
}