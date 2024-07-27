use super::value::Value;
use std::{collections::hash_map::Entry, iter::Sum, ops::{Add, Div, Index, IndexMut, Mul, Rem, Shl, Shr, Sub}, sync::{Arc, Mutex, MutexGuard}};

macro_rules! arr_op {
    ($a:expr, $x:tt, $b:expr) => {
        let mut res: Vec<Value> = vec![];
        let mut i = 0;
        while i < $a.len() { if $b.len() > i { res.push($a[i].clone() $x $b[i].clone()); } else { res.push($a[i].clone()); } i += 1; }
        if $b.len() > $a.len() { res.append(&mut $b[$a.len()..].to_vec()) }
        return Value::Arr(res);
    };
}
macro_rules! join_arr {
    ($sep:expr, $arr:expr) => {
        return Value::String($arr.iter().map(|i| format!("{i}")).collect::<Vec<String>>().join($sep));
    };
}

// basic ops

impl Add for Value {
    type Output = Self;
    fn add (self, rhs: Self) -> Self {
        match &self {
            Value::Arr(a) => {
                match rhs {
                    Value::Arr(b) => { arr_op!(a, +, b); }
                    // Value::Arr(mut b) => { let mut a = a.clone(); a.append(&mut b); return Value::Arr(a) }
                    Value::String(b) => { join_arr!(&b, a); }
                    _ => {}
                }
            }

            Value::String(a) => {
                match rhs {
                    Value::String(b) => { return Value::String(format!("{a}{b}")) }
                    Value::Arr(b) => { join_arr!(&a, b); }
                    _ => {}
                }
            }
            _ => {}
        }

        Value::Number(self.cast_float() + rhs.cast_float())
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        match &self {
            Value::Arr(a) => {
                match rhs {
                    Value::Arr(b) => { arr_op!(a, -, b); }
                    _ => {}
                }
            }

            Value::String(a) => {
                match rhs {
                    Value::String(b) => { return Value::String(a.replace(&b, "")) }
                    _ => {}
                }
            }

            _ => {}
        }

        Value::Number(self.cast_float() - rhs.cast_float())
    }    
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        match &self {
            Value::Arr(a) => {
                match rhs {
                    Value::Arr(b) => { arr_op!(a, *, b); }
                    _ => {}
                }
            }

            _ => {}
        }

        Value::Number(self.cast_float() * rhs.cast_float())
    }    
}

impl Div for Value {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        match &self {
            Value::Arr(a) => {
                match rhs {
                    Value::Arr(b) => { arr_op!(a, /, b); }
                    _ => {}
                }
            }

            Value::String(a) => {
                match rhs {
                    Value::String(b) => { return Value::Arr(a.split(&b).map(|i| Value::String(i.to_string())).collect::<Vec<Value>>()) }
                    _ => {}
                }
            }

            _ => {}
        }

        Value::Number(self.cast_float() / rhs.cast_float())
    }    
}

impl Rem for Value {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        match &self {
            Value::Arr(a) => {
                match rhs {
                    Value::Arr(b) => { arr_op!(a, %, b); }
                    _ => {}
                }
            }

            _ => {}
        }

        Value::Number(self.cast_float() % rhs.cast_float())
    }    
}

impl Shl for Value {
    type Output = Self;
    fn shl(self, rhs: Self) -> Self::Output {
        match &self {
            Value::Arr(a) => {
                match rhs {
                    Value::Arr(b) => { arr_op!(a, <<, b); }
                    _ => {}
                }
            }

            _ => {}
        }

        Value::Number(((self.cast_float() as i64) << (rhs.cast_float() as i64)) as f64)
    }
}

impl Shr for Value {
    type Output = Self;
    fn shr(self, rhs: Self) -> Self::Output {
        match &self {
            Value::Arr(a) => {
                match rhs {
                    Value::Arr(b) => { arr_op!(a, >>, b); }
                    _ => {}
                }
            }

            _ => {}
        }

        Value::Number(((self.cast_float() as i64) >> (rhs.cast_float() as i64)) as f64)
    }
}


// accessing and changing values in containers (string, array, dictionary)

impl Index<Value> for Vec<Value> {
    type Output = Value;

    fn index(&self, index: Value) -> &Self::Output {
        match index {
            Value::Arr(x) => {
                let mut result = vec![];
                for i in x { result.push(self[i].clone()); }

                let r = Box::new(Value::Arr(result));
                Box::leak(r)
            }
            Value::Number(x) => {
                if x < 0.0 { &self[(self.len() as f64 + x) as usize] }
                else { &self[x as usize] }
            },
            _ => { &Value::Undefined }
        }
    }
}

impl IndexMut<Value> for Vec<Value> {
    fn index_mut(&mut self, index: Value) -> &mut Self::Output {
        match index {
            Value::Number(x) => &mut self[x as usize],
            _ => { Box::leak(Box::new(Value::Undefined)) }
        }
    }
}

impl Index<Value> for String {
    type Output = Value;

    fn index(&self, index: Value) -> &Self::Output {
        let v = self.chars().collect::<Vec<char>>();
        match index {
            Value::Arr(x) => {
                let mut result = vec![];
                for i in x { result.push(v[i.cast_float() as usize]); }

                let r = Box::new(Value::String(result.iter().collect::<String>()));
                Box::leak(r)
            }
            Value::Number(x) => {
                let r = Box::new(Value::String(v[x as usize].to_string()));
                Box::leak(r)
            },
            _ => { &Value::Undefined }
        }
    }
}

impl Index<Value> for Value {
    type Output = Value;

    fn index(&self, index: Value) -> &Self::Output {
        match self {
            Value::Arr(x) => return &x[index],
            Value::String(x) => return &x[index],
            Value::Dict(x) => {
                if let Some(y) = x.get(&index) { return y; }
            }
            _ => {}
        }
        &Value::Undefined
    }
}

impl IndexMut<Value> for Value {
    fn index_mut(&mut self, index: Value) -> &mut Self::Output {
        match self {
            Value::Arr(x) => return &mut x[index],
            // Value::String(x) => return &mut x[index],
            Value::Dict(x) => {
                return match x.entry(index) {
                    Entry::Occupied(o) => o.into_mut(),
                    Entry::Vacant(v) => v.insert(Value::Undefined)
                }
            }
            _ => {}
        }

        Box::leak(Box::new(Value::Undefined))
    }
}

pub fn set (d: Arc<Mutex<Value>>, index: Value, v: Value) {
    match index {
        Value::Arr(x) => {
            if let Value::Dict(_) = d.lock().unwrap().clone() { d.lock().unwrap()[Value::Arr(x)] = v; return; }
            for i in x { set(d.clone(), i, v.clone()); }
        }
        _ => {
            if let Value::Arr(mut a) = d.lock().unwrap().clone() {
                if let Value::Number(x) = index {
                    let x = x as usize;
                    if x >= a.len() {
                        a.resize_with(x+1, || Value::Undefined);
                    }
                }
            }

            d.lock().unwrap()[index] = v;
        }
    }
}

// more array operations

impl Sum<Value> for Value {
    fn sum<I: Iterator<Item = Value>>(iter: I) -> Self {
        iter.fold(Value::Undefined, |a, b| a + b)
    }
}

// references and simulated memory

// pub fn assign_by_ref (memory: &mut ReferenceMap<String, Value>, r: Value, v: Value) {
//     if let Value::Ref(x) = r {
//         memory.insert(x, v);
//     }
// }

// impl PartialEq for Arc<Mutex<Value>> {
//     fn eq(&self, _: &Arc<Mutex<Value>>) -> bool {
//         false
//     }
// }