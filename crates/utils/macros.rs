#[doc(hidden)]
#[macro_export]
macro_rules! unimpl {
    ($name:expr) => {
        log::debug!("{} is not implemented", $name);
    };
}
#[doc(inline)]
pub use unimpl;
