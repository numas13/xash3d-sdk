use core::slice;

pub fn array_from_slice<T: Copy + Default, const N: usize>(slice: &[T]) -> [T; N] {
    let mut arr = [T::default(); N];
    arr.copy_from_slice(&slice[..N]);
    arr
}

/// Forms a slice from a pointer and a length or returns an empty slice if the pointer is null.
///
/// The `len` argument is the number of elements, not the number of bytes.
///
/// # Safety
///
/// See [core::slice::from_raw_parts].
pub unsafe fn slice_from_raw_parts_or_empty<'a, T>(data: *const T, len: usize) -> &'a [T] {
    if data.is_null() {
        &[]
    } else {
        unsafe { slice::from_raw_parts(data, len) }
    }
}

/// Performs the same functionality as [slice_from_raw_parts_or_empty], except
/// that a mutable slice is returned.
///
/// # Safety
///
/// See [core::slice::from_raw_parts_mut].
pub unsafe fn slice_from_raw_parts_or_empty_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T] {
    if data.is_null() {
        &mut []
    } else {
        unsafe { slice::from_raw_parts_mut(data, len) }
    }
}
