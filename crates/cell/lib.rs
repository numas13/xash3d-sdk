#![no_std]

use core::{
    cell::OnceCell,
    ops::{Deref, DerefMut},
};

/// A tiny wrapper for single-threaded mutable statics.
///
/// The cell implements [core::marker::Sync] and can be stored in static.
///
/// # Safety
///
/// Only for single-threaded use.
pub struct Sync<T>(T);

/// SAFETY: the engine is single-threaded.
unsafe impl<T> core::marker::Sync for Sync<T> {}

impl<T> Sync<T> {
    /// # Safety
    ///
    /// This is not thread-safe! Make sure the cell will not be used in multiple threads.
    pub const unsafe fn new(data: T) -> Self {
        Self(data)
    }
}

impl<T> Deref for Sync<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Sync<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A tiny wrapper for [core::cell::OnceCell].
///
/// The cell implements [core::marker::Sync] and can be stored in static.
///
/// # Safety
///
/// Only for single-threaded use.
pub struct SyncOnceCell<T>(OnceCell<T>);

/// SAFETY: the engine is single-threaded.
unsafe impl<T> core::marker::Sync for SyncOnceCell<T> {}

impl<T> SyncOnceCell<T> {
    /// # Safety
    ///
    /// This is not thread-safe! Make sure the cell will not be used in multiple threads.
    pub const unsafe fn new() -> Self {
        Self(OnceCell::new())
    }
}

impl<T> Deref for SyncOnceCell<T> {
    type Target = OnceCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SyncOnceCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
