pub(super) const LEAST_PRIORITY: Priority = Priority(1);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Priority(usize);
impl Priority {
    pub(super) const fn new(priority: usize) -> Self {
        assert!(priority <= LEAST_PRIORITY.as_usize(), "Invalid priority");

        Self(priority)
    }

    pub(super) const fn as_usize(self) -> usize {
        self.0
    }
}
