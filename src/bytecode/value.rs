use std::collections::HashSet;
use crate::lexer::Token;
use super::gen::popv;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64), String(String), Boolean(bool),

    Raw(Box<Value>), ParseModes(HashSet<String>) /* value for binds */,
    
    Var(String, Box<Value>), Get(String), TCall(String, usize), Call(String, Vec<Value>),
    
    NumOp(Box<Value>, Box<Value>, String), LogOp(Box<Value>, Box<Value>, String), Pow(Box<Value>, Box<Value>),
    Not(Box<Value>),
    If(Box<Value>, Vec<Value>), ElseIf(Box<Value>, Vec<Value>), Else(Vec<Value>), PassedIf, FailedIf,
    Loop(Vec<Value>), Break, Continue,
    Do(Vec<Token>, i32), Fn(String, Vec<String>, Vec<Value>), Array(Vec<Value>), Dict(Vec<Value>, Vec<Value>), Mov(String), Println(Box<Value>),
    Undefined, Block(Vec<Value>),
    Pick(Box<Value>, Box<Value>), Set(Box<Value>, Box<Value>, Box<Value>),
    Type(Box<Value>), Push(Box<Value>, Box<Value>),

    Ref(Box<Value>), RefAssign(String, Box<Value>),
    
    RustBinding(Vec<Value>), RustReturnableBinding(Vec<Value>)
}

impl Value {

    pub fn is_static (&self) -> bool {
        match self {
            Value::Number(_) | Value::String(_) | Value::Boolean(_) | Value::Undefined => true,
            Value::Array(v) | Value::Block(v) => is_static_array(v),
            _ => false
        }
    }

    pub fn cast_float_static (self) -> Option<f64> {
        match self {
            Value::Number(x) => return Some(x),
            Value::Array(x) => {
                if is_static_array(&x) {
                    return Some(x.len() as f64);
                }
            }
            Value::Block(mut x) => {
                if is_static_array(&x) {
                    if let Some(v) = popv(&mut x) {
                        return v.cast_float_static()
                    }
                }
            }
            Value::String(x) => return Some(x.len() as f64),
            Value::Boolean(x) => return Some(x as i64 as f64),
            _ => {}
        }
        None
    }

    pub fn cast_bool_static (self) -> Option<bool> {
        match self {
            Value::Number(x) => return Some(x > 0.0),
            Value::Array(x) => {
                if is_static_array(&x) {
                    return Some(x.len() > 0);
                }
            },
            Value::Block(mut x) => {
                if is_static_array(&x) {
                    if let Some(v) = popv(&mut x) {
                        return v.cast_bool_static()
                    }
                }
            },
            Value::String(x) => return Some(x.len() > 0),
            Value::Boolean(x) => return Some(x),
            _ => {}
        }
        None
    }

    pub fn process (self) -> Value {
        macro_rules! lpp {
            ($a:expr, $op:tt, $b:expr) => {
                return Value::Boolean($a.cast_float_static().unwrap() $op $b.cast_float_static().unwrap())
            };
        }

        match &self {
            Value::LogOp(a, b, op) => {
                let a = *a.clone(); let b = *b.clone();
                if a.is_static() && b.is_static() {
                    match op.as_str() {
                        ">" => lpp!(a, >, b),
                        "<" => lpp!(a, <, b),
                        ">=" => lpp!(a, >=, b),
                        "<=" => lpp!(a, <=, b),
                        "=" => return Value::Boolean(a == b),
                        "!=" => return Value::Boolean(a != b),
                        "||" => return Value::Boolean(a.cast_bool_static().unwrap() || b.cast_bool_static().unwrap()),
                        "&&" => return Value::Boolean(a.cast_bool_static().unwrap() && b.cast_bool_static().unwrap()),
                        _ => {}
                    }                    
                }
            },
            Value::Not(x) => { if (*x).is_static() { return Value::Boolean(!(x.clone().cast_bool_static().unwrap())) } }
            Value::NumOp(a, b, op) => {
                let a = *a.clone(); let b = *b.clone();
                if a.is_static() && b.is_static() {
                    let a = if let Value::Block(mut v) = a.clone() {
                        if let Some(x) = popv(&mut v) { x }
                        else { a }
                    } else { a };
                    let b = if let Value::Block(mut v) = a.clone() {
                        if let Some(x) = popv(&mut v) { x }
                        else { b }
                    } else { b };
                    match op.as_str() {
                        "+" => { return a + b },
                        "-" => { return a - b },
                        "/" => {},
                        "*" => {},
                        "%" => {},
                        // ">>" => return Value::Number((a.cast_float_static() as i64) ),
                        // "<<" => return Value::Boolean(a != b),
                        _ => {}
                    }                    
                }
            }
            _ => {}
        }
        self
    }

}

pub fn is_static_array (a: &Vec<Value>) -> bool {
    for i in a {
        if !i.is_static() { return false; }
    }

    true
}