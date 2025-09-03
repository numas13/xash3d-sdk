use core::{iter, ptr};

use crate::raw::TEMPENTITY;

pub struct TempEntityList {
    head: *mut *mut TEMPENTITY,
    free: *mut *mut TEMPENTITY,
}

impl TempEntityList {
    /// Creates a `TempEntityList` directly from pointers.
    pub(crate) unsafe fn from_raw_parts(
        head: *mut *mut TEMPENTITY,
        free: *mut *mut TEMPENTITY,
    ) -> Self {
        Self { head, free }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut TEMPENTITY) -> bool,
    {
        unsafe {
            let mut prev = ptr::null_mut();
            let mut temp = *self.head;
            while !temp.is_null() {
                let next = (*temp).next;
                if f(&mut *temp) {
                    // keep item
                    prev = temp;
                } else {
                    // remove item
                    (*temp).next = *self.free;
                    *self.free = temp;
                    if prev.is_null() {
                        // remove from head
                        *self.head = next;
                    } else {
                        (*prev).next = next;
                    }
                }
                temp = next;
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TEMPENTITY> {
        let mut temp = unsafe { *self.head };
        iter::from_fn(move || {
            if !temp.is_null() {
                let ret = unsafe { &mut *temp };
                temp = ret.next;
                Some(ret)
            } else {
                None
            }
        })
    }
}
