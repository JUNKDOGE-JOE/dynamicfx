pub(crate) struct Checkouts<F: FnMut(u32) -> Result<(), after_effects::Error>> {
    ids: Vec<u32>,
    checkin: F,
}

impl<F: FnMut(u32) -> Result<(), after_effects::Error>> Checkouts<F> {
    pub(crate) fn new(checkin: F) -> Self { Self { ids: Vec::new(), checkin } }
    pub(crate) fn record(&mut self, id: u32) { self.ids.push(id); }
    fn release(&mut self) -> Result<(), after_effects::Error> {
        let mut result = Ok(());
        while let Some(id) = self.ids.pop() {
            let next = (self.checkin)(id);
            if result.is_ok() { result = next; }
        }
        result
    }
    pub(crate) fn finish(mut self) -> Result<(), after_effects::Error> { self.release() }
}

impl<F: FnMut(u32) -> Result<(), after_effects::Error>> Drop for Checkouts<F> {
    fn drop(&mut self) { let _ = self.release(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelled_external_checkout_releases_only_successful_checkouts() {
        let mut released = Vec::new();
        let result = (|| {
            let mut guard = Checkouts::new(|id| { released.push(id); Ok(()) });
            guard.record(0);
            guard.record(1);
            Err::<(), _>(after_effects::Error::InterruptCancel)
        })();
        assert_eq!(result, Err(after_effects::Error::InterruptCancel));
        assert_eq!(released, [1, 0]);
    }
    #[test]
    fn checkin_error_does_not_skip_remaining_ids_or_double_release() {
        let mut released = Vec::new();
        let mut guard = Checkouts::new(|id| { released.push(id); Err(after_effects::Error::Generic) });
        guard.record(0);
        guard.record(4096);
        guard.record(2);
        assert_eq!(guard.finish(), Err(after_effects::Error::Generic));
        assert_eq!(released, [2, 4096, 0]);
    }
    #[test]
    fn unwind_releases_empty_successful_checkout_too() {
        let mut released = Vec::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut guard = Checkouts::new(|id| { released.push(id); Ok(()) });
            guard.record(0);
            panic!("frame aborted");
        }));
        assert!(result.is_err());
        assert_eq!(released, [0]);
    }
}
