use core::{cell::Cell, fmt, ops::Deref, ptr::NonNull};

#[derive(Debug)]
pub struct BorrowRef {
    lock: Cell<bool>,
}

impl BorrowRef {
    pub const fn new() -> Self {
        Self {
            lock: Cell::new(false),
        }
    }

    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// * `value` must be non-null.
    /// * The memory referenced by the returned wrapper must not be mutated for the duration
    ///   of lifetime 'b.
    pub unsafe fn borrow<'b, T: 'b>(&'b self, value: *mut T) -> Ref<'b, T> {
        assert!(!value.is_null());
        assert!(!self.lock.replace(true));
        Ref::new(value, self)
    }
}

impl Default for BorrowRef {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Ref<'b, T: 'b> {
    value: NonNull<T>,
    borrow: &'b BorrowRef,
}

impl<'b, T> Ref<'b, T> {
    fn new(value: *mut T, borrow: &'b BorrowRef) -> Self {
        Self {
            value: NonNull::new(value).unwrap(),
            borrow,
        }
    }
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        self.borrow.lock.set(false);
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<T: fmt::Display> fmt::Display for Ref<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrow() {
        let value = BorrowRef::new();
        let a = unsafe { value.borrow(&mut 0) };
        drop(a);
        let b = unsafe { value.borrow(&mut 0) };
        drop(b);
    }

    #[should_panic]
    #[test]
    fn borrow_fail() {
        let value = BorrowRef::new();
        let _a = unsafe { value.borrow(&mut 0) };
        let _b = unsafe { value.borrow(&mut 0) };
    }
}
