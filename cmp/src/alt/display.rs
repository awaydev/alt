use super::value::Value;
use std::fmt;

impl fmt::Display for Value {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Number(r) => write!(f, "{r}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Dict(d) => write!(f, "{d:?}"),
            Value::Arr(x) => write!(f, "{x:?}"),
            Value::Ref(x) => write!(f, "{}", x.clone()),
            _ => write!(f, "undefined")
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return match self {
                Value::String(s) => write!(f, "{s:?}"),
                Value::Number(_) | Value::Boolean(_) => write!(f, "{self}"),
                Value::Dict(d) => write!(f, "{d:#?}"),
                Value::Arr(x) => write!(f, "{x:#?}"),
                Value::Ref(x) => write!(f, "&{:#?}", x.clone()),
                _ => write!(f, "undefined")
            }
        }
        match self {
            Value::String(s) => write!(f, "{s:?}"),
            Value::Number(_) | Value::Boolean(_) => write!(f, "{self}"),
            Value::Dict(d) => write!(f, "{d:?}"),
            Value::Arr(x) => write!(f, "{x:?}"),
            Value::Ref(x) => write!(f, "{:?}", x.clone()),
            _ => write!(f, "undefined")
        }
    }
}