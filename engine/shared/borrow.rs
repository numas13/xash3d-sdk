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

    pub fn borrow<T>(&self, value: *mut T) -> Ref<T> {
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
        let a = value.borrow(&mut 0);
        drop(a);
        let b = value.borrow(&mut 0);
        drop(b);
    }

    #[should_panic]
    #[test]
    fn borrow_fail() {
        let value = BorrowRef::new();
        let _a = value.borrow(&mut 0);
        let _b = value.borrow(&mut 0);
    }
}
