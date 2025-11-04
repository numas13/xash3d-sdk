pub type EnvLightning = crate::env_beam::EnvBeam;

#[doc(hidden)]
#[macro_export]
macro_rules! export_env_lightning {
    () => {
        $crate::export_entity!(env_lightning, $crate::env_lightning::EnvLightning);
    };
}
#[doc(inline)]
pub use export_env_lightning as export;
