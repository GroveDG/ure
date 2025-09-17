use std::{marker::PhantomData, ops::Range};



#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidIndex<'a> {
    inner: usize,
    _marker: PhantomData<&'a ()>,
}
impl Into<usize> for ValidIndex<'_> {
    fn into(self) -> usize {
        self.inner
    }
}
impl ValidIndex<'_> {
    unsafe fn from_index<'a>(index: usize) -> ValidIndex<'a> {
        ValidIndex {
            inner: index,
            _marker: PhantomData::default(),
        }
    }
    pub fn inner(&self) -> usize {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub struct ValidRange<'a> {
    pub(super) inner: Range<usize>,
    pub(super) _marker: PhantomData<&'a ()>,
}
impl<'a> ValidRange<'a> {
    pub fn validate(&self, index: usize) -> Option<ValidIndex<'a>> {
        if self.inner.contains(&index) {
            Some(unsafe { ValidIndex::from_index::<'a>(index) })
        } else {
            None
        }
    }
}
impl PartialEq<usize> for ValidIndex<'_> {
    fn eq(&self, other: &usize) -> bool {
        self.inner.eq(other)
    }
}
impl PartialOrd<usize> for ValidIndex<'_> {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(other)
    }
}
impl<'a> Iterator for ValidRange<'a> {
    type Item = ValidIndex<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.inner.next()?;
        Some(unsafe { ValidIndex::from_index::<'a>(n) })
    }
}
