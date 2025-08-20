use core::mem::MaybeUninit;

pub trait UnsyncGlobal: Sized {
    fn global_as_mut_ptr() -> *mut MaybeUninit<Self>;

    /// # Safety
    ///
    /// Calling this when the global object is not yet initialized causes undefined behavior.
    unsafe fn global_assume_init_ref<'a>() -> &'a Self {
        unsafe { (*Self::global_as_mut_ptr()).assume_init_ref() }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_unsync_global {
    ($name:ty) => {
        impl $crate::export::UnsyncGlobal for $name {
            fn global_as_mut_ptr() -> *mut core::mem::MaybeUninit<Self> {
                static mut INSTANCE: core::mem::MaybeUninit<$name> =
                    core::mem::MaybeUninit::uninit();
                unsafe { core::ptr::addr_of_mut!(INSTANCE) }
            }
        }
    };
}
pub use impl_unsync_global;
