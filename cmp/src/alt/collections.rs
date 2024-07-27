use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Eq;
use std::iter::FromIterator;
use crate::Value;

impl Eq for Value {}
impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Number(n) => { (n.clone() as i64).hash(state) }
            Value::Arr(n) => { n.hash(state); }
            Value::String(n) => { n.hash(state); }
            Value::Boolean(n) => { n.hash(state); }
            Value::Dict(n) => { n.iter().collect::<Vec<(&Value, &Value)>>().hash(state); }
            Value::Undefined | Value::Empty | Value::Ref(_) => { 0.hash(state); }
        }
    }
}

pub fn dict (keys: Vec<Value>, values: Vec<Value>) -> HashMap<Value, Value> {
    assert!(keys.len() == values.len(), "failed to create hashmap: shapes [{}] and [{}] don't match.", keys.len(), values.len());
    HashMap::from_iter(keys.iter().enumerate().map(|(index, _)| (keys[index].clone(), values[index].clone())))
}