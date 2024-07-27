use crate::Value;

pub fn pop (stack: &mut Vec<Value>) -> Value {
    return stack.pop().expect("ERROR: stack underflow")
}

pub fn push (stack: &mut Vec<Value>, value: Value) {
    if Value::Empty != value {
        return stack.push(value)
    }
}