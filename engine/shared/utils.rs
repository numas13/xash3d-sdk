pub fn array_from_slice<T: Copy + Default, const N: usize>(slice: &[T]) -> [T; N] {
    let mut arr = [T::default(); N];
    arr.copy_from_slice(&slice[..N]);
    arr
}
