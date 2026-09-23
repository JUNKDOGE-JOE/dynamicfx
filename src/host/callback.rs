use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    static ACTIVE: RefCell<HashSet<usize>> = RefCell::new(HashSet::new());
}

pub(crate) struct HostCallScope {
    instance: usize,
}

impl HostCallScope {
    pub(crate) fn enter<T>(instance: &T) -> Option<Self> {
        let instance = std::ptr::from_ref(instance) as usize;
        ACTIVE.with(|active| active.borrow_mut().insert(instance))
            .then(|| Self { instance })
    }
}

impl Drop for HostCallScope {
    fn drop(&mut self) {
        ACTIVE.with(|active| active.borrow_mut().remove(&self.instance));
    }
}
