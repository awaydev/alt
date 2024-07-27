use std::sync::{Arc, Mutex, MutexGuard};
use crate::Value;

#[derive(Debug, Clone)]
pub struct Ref {
    arc: Arc<Mutex<Value>>
}

impl PartialEq for Ref {
    fn eq(&self, other: &Self) -> bool {
        let a = self.arc.lock().unwrap().clone();
        let b = other.arc.lock().unwrap().clone();
        a == b
    }
}

impl Ref {

    pub fn new (arc: Arc<Mutex<Value>>) -> Self {
        Self { arc }
    }

    pub fn get_arc (&self) -> Arc<Mutex<Value>> {
        self.arc.clone()
    }

    pub fn clone_ref (&self) -> Ref {
        Ref::new(self.arc.clone())
    }

    pub fn lock (&self) -> MutexGuard<'_, Value> {
        self.arc.lock().unwrap()
    }

    pub fn clone (&self) -> Value {
        self.lock().clone()
    }

}

pub type Covered = Ref;

#[macro_export]
macro_rules! nvar {
    ($v:expr) => {
        // Arc::new(Mutex::new($v))
        Ref::new(Arc::new(Mutex::new($v)))
    };
}