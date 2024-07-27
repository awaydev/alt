use std::{cell::RefCell, collections::HashMap, sync::{Arc, Mutex}};
use super::r#ref::Ref;

#[derive(PartialEq, Clone)]
pub enum Value {
    String(String), Number(f64), Boolean(bool), Arr(Vec<Value>), Dict(HashMap<Value, Value>), Ref(Ref), Undefined, Empty
}

impl Value {
    pub fn cast_float (&self) -> f64 {
        match self {
            Value::String(x) => x.len() as f64,
            Value::Number(x) => *x,
            Value::Boolean(x) => { if *x { return 1.0 } else { return 0.0 } }
            Value::Arr(x) => return x.len() as f64,
            Value::Dict(x) => x.len() as f64,
            Value::Ref(x) => x.clone().cast_float(),
            _ => 0.0
        }
    }

    pub fn cast_int (&self) -> i64 {
        match self {
            Value::String(x) => x.len() as i64,
            Value::Number(x) => *x as i64,
            Value::Boolean(x) => *x as i64,
            Value::Arr(x) => x.len() as i64,
            Value::Dict(x) => x.len() as i64,
            Value::Ref(x) => x.clone().cast_int(),
            _ => 0
        }
    }

    pub fn cast_bool (&self) -> bool {
        match self {
            Value::Boolean(x) => *x,
            Value::Number(x) => *x > 0.0,
            Value::Arr(x) => x.len() > 0,
            Value::String(x) => x.len() > 0,
            Value::Ref(x) => x.clone().cast_bool(),
            _ => false
        }
    }

    pub fn cast_string (&self) -> String {
        format!("{self}")
    }

    pub fn cast_vec (self) -> Vec<Value> {
        match self {
            Value::Number(x) => vec![Value::Number(0.0); x as usize],
            Value::Arr(x) => x,
            Value::String(x) => x.chars().map(|i| Value::String(String::from(i))).collect::<Vec<Value>>(),
            Value::Ref(x) => x.clone().cast_vec(),
            _ => vec![]
        }
    }

    pub fn cast_type (self) -> String {
        match self {
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Arr(_) => "array",
            Value::Boolean(_) => "boolean",
            Value::Dict(_) => "dictionary",
            Value::Ref(x) => return format!("reference>{}", x.clone().cast_type()),
            _ => "undefined"
        }.to_string()
    }

    pub fn push (&mut self, value: Value) {
        match self {
            Value::Arr(x) => x.push(value),
            Value::Ref(x) => x.lock().push(value),
            _ => {}
        }
    }

    pub fn inc (&mut self) {
        match self {
            Value::String(a) => { *a = a.to_uppercase(); }
            Value::Number(a) => { *a += 1.0; }
            Value::Arr(a) => { a.iter_mut().for_each(|x| x.inc()); }
            Value::Ref(a) => { a.lock().inc(); }
            _ => {}
        }
    }

    pub fn dec (&mut self) {
        match self {
            Value::String(a) => { *a = a.to_lowercase(); }
            Value::Number(a) => { *a -= 1.0; }
            Value::Arr(a) => { a.iter_mut().for_each(|x| x.dec()); }
            Value::Ref(a) => { a.lock().dec(); }
            _ => {}
        }
    }

    pub fn sum (&self) -> Value {
        match self {
            Value::Arr(a) => { a.iter().cloned().sum::<Value>() }
            Value::Ref(a) => { a.lock().sum() }
            _ => { Value::Number(0.0) }
        }
    }
    
    pub fn concat (self, v: Value) -> Value {
        let mut a = self.cast_vec();
        a.append(&mut v.cast_vec());
        Value::Arr(a)
    }

    pub fn flat (self) -> Value {
        let a = self.cast_vec();
        let mut b: Vec<Value> = vec![];
        for i in a { if let Value::Arr(mut x) = i { b.append(&mut x); continue; } b.push(i); }
        Value::Arr(b)
    }
}