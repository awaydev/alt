use std::ops::{Add, Index, Sub};
use super::value::Value;

/* This is implimentation of operators for static values. It's needed to do stuff at compile-time.
Maybe it's too bloated, but it is what it is, lol. (literally cutted copy of module for Rust target) */

macro_rules! arr_op {
    ($a:expr, $x:tt, $b:expr) => {
        let mut res: Vec<Value> = vec![];
        let mut i = 0;
        while i < $a.len() { if $b.len() > i { res.push($a[i].clone() $x $b[i].clone()); } else { res.push($a[i].clone()); } i += 1; }
        if $b.len() > $a.len() { res.append(&mut $b[$a.len()..].to_vec()) }
        return Value::Array(res);
    };
}

macro_rules! join_arr {
    ($sep:expr, $arr:expr) => {
        return Value::String($arr.iter().map(|i| format!("{i}")).collect::<Vec<String>>().join($sep));
    };
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match &self {
            Value::Array(a) => {
                match rhs {
                    Value::Array(b) => { arr_op!(a, +, b); },
                    Value::Number(b) => { return Value::Number(a.len() as f64 + b) }
                    Value::String(b) => { join_arr!(&b, a); }
                    _ => {}
                }
            }
            Value::Number(a) => {
                match rhs {
                    Value::Number(b) => { return Value::Number(a + b) }
                    Value::Array(b) => { return Value::Number(a + b.len() as f64) }
                    Value::String(b) => { return Value::Number(a + b.len() as f64) }
                    _ => {}
                }
            }
            Value::String(a) => {
                match rhs {
                    Value::Number(b) => { return Value::Number(a.len() as f64 + b) }
                    Value::String(b) => { return Value::String(format!("{a}{b}")) }
                    Value::Array(b) => { join_arr!(&a, b); }
                    _ => {}
                }
            }
            _ => {}
        }
        Value::NumOp(Box::new(self), Box::new(rhs), String::from("+"))
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match &self {
            Value::Array(a) => {
                match rhs {
                    Value::Array(b) => { arr_op!(a, -, b); },
                    Value::Number(b) => { return Value::Number(a.len() as f64 - b) }
                    _ => {}
                }
            }
            Value::Number(a) => {
                match rhs {
                    Value::Number(b) => { return Value::Number(a - b) }
                    Value::Array(b) => { return Value::Number(a - b.len() as f64) }
                    Value::String(b) => { return Value::Number(a - b.len() as f64) }
                    _ => {}
                }
            }
            Value::String(a) => {
                match rhs {
                    Value::Number(b) => { return Value::Number(a.len() as f64 - b) }
                    Value::String(b) => { return Value::String(a.replace(&b, "")) }
                    _ => {}
                }
            }
            _ => {}
        }
        Value::NumOp(Box::new(self), Box::new(rhs), String::from("-"))
    }
}


impl Index<Value> for Value {
    type Output = Value;

    fn index(&self, index: Value) -> &Self::Output {
        match self {
            Value::Array(x) => {
                match index {
                    Value::Number(y) => {
                        let mut y = y as isize;
                        if y < 0 { y = x.len() as isize + y; }
                        if y >= x.len() as isize { return &Value::Undefined }
                        return &x[y as usize]
                    }
                    Value::Array(y) => {
                        let mut result = vec![];
                        for i in y { result.push(self[i].clone()); }
                        return Box::leak(Box::new(Value::Array(result)))
                    }
                    _ => {}
                }
            },
            Value::String(x) => {
                let x: Vec<char> = x.chars().collect();
                match index {
                    Value::Number(y) => {
                        let mut y = y as isize;
                        if y < 0 { y = x.len() as isize + y; }
                        if y >= x.len() as isize { return &Value::Undefined }
                        return Box::leak(Box::new(Value::String(x[y as usize].to_string())))
                    },
                    Value::Array(y) => {
                        let mut result = vec![];
                        for i in y { result.push(self[i].clone()); }
                        return Box::leak(Box::new(Value::Array(result)))
                    }
                    _ => {}
                }
            },
            
            _ => {}
        }
        &Value::Undefined
    }
}

// new ops coming soon...