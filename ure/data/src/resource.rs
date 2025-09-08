use std::{sync::Weak, sync::Arc};

use parking_lot::Mutex;

pub struct Resource<T, F = fn() -> T> {
    weak: Mutex<Weak<T>>,
    f: F,
}
impl<T, F: Fn() -> T> Resource<T, F> {
    pub const fn new(f: F) -> Self {
        Self {
            weak: Mutex::new(Weak::new()),
            f,
        }
    }
    pub fn load(&'static self) -> Arc<T> {
        let mut lock = self.weak.lock();
        if let Some(arc) = lock.upgrade() {
            return arc;
        }
        let arc = Arc::new((self.f)());
        *lock = Arc::downgrade(&arc);
        arc
    }
}