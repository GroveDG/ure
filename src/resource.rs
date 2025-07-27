use std::sync::{Arc, Weak};

use parking_lot::Mutex;

pub struct Resource<T: Load> {
    source: T::Source,
    weak: Mutex<Weak<T>>,
}
impl<T: Load> Resource<T> {
    const fn new(source: T::Source) -> Self {
        Self {
            source,
            weak: Mutex::new(Weak::new()),
        }
    }
    fn load(&'static self) -> Arc<T> {
        let mut lock = self.weak.lock();
        if let Some(arc) = lock.upgrade() {
            return arc;
        }
        let arc = Arc::new(T::load(&self.source));
        *lock = Arc::downgrade(&arc);
        arc
    }
}

pub trait Load {
    type Source;

    fn load(source: &Self::Source) -> Self;
}