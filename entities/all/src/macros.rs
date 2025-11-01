#[doc(hidden)]
#[macro_export]
macro_rules! stub {
    () => {};
}

#[allow(unused_macros)]
macro_rules! define_export {
    (
        $( #[$attr:meta] )*
        $macro:ident $(as $alias:ident)? $( if $( $feature:literal )or+ )? {
            $( $name:ident = $( $path:ident )::+ ),+ $(,)?
        }
    ) => {
        $( #[$attr] )*
        $( #[cfg(any($( feature = $feature ),+))] )?
        #[doc(hidden)]
        #[macro_export]
        macro_rules! $macro {
            () => {
                $( $crate::export_entity!($name, $crate::$( $path )::+); )+
            };
        }

        $( #[cfg(any($(feature = $feature ),+))] )?
        #[doc(inline)]
        pub use $macro $(as $alias)?;

        $( #[cfg(any($(feature = $feature ),+))] )?
        #[doc(hidden)]
        pub use $macro as __export;

        $(
            #[cfg(not(any($(feature = $feature ),+)))]
            #[doc(hidden)]
            pub use stub as __export;
        )?
    };
}

macro_rules! import {
    (
        $(
            $( #[$attr:meta] )*
            use $( $dep:ident )::+ as $name:ident $( if $( $feature:literal )or+ )?;
        )*
    ) => {
        $(
            $( #[$attr] )*
            $( #[cfg(any($( feature = $feature ),+))] )?
            pub mod $name {
                pub use $( $dep )::+::*;

                #[doc(inline)]
                pub use $( $dep )::+::export;

                #[doc(hidden)]
                pub use $( $dep )::+::export as __export;
            }

            $(
                #[cfg(not(any($( feature = $feature ),+)))]
                #[doc(hidden)]
                pub mod $name {
                    pub use crate::stub as __export;
                }
            )?
        )*
    };
}

macro_rules! import_with_export {
    (
        $export_name:ident ;
        $(
            $( #[$attr:meta] )*
            use $( $dep:ident )::+ as $name:ident $( if $( $feature:literal )or+ )?;
        )*
    ) => {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! $export_name {
            () => {
                $( $crate::$name::__export!(); )*
            };
        }

        import! {
            $(
                $( #[$attr] )*
                use $( $dep )::+ as $name $( if $( $feature )or+ )?;
            )*
        }
    };
}

macro_rules! define {
    (
        $(
            $( #[$attr:meta] )*
            mod $name:ident if $( $feature:literal )or+;
        )*
    ) => {
        $(
            $( #[$attr] )*
            #[cfg(any($( feature = $feature ),+))]
            pub mod $name;

            #[cfg(not(any($( feature = $feature ),+)))]
            #[doc(hidden)]
            pub mod $name {
                pub use crate::stub as __export;
            }
        )*
    };
}

macro_rules! define_with_export {
    (
        $export_name:ident;
        $(
            $( #[$attr:meta] )*
            mod $name:ident if $( $feature:literal )or+;
        )*
    ) => {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! $export_name {
            () => {
                $( $crate::$name::__export!(); )*
            };
        }

        define! {
            $(
                $( #[$attr] )*
                mod $name if $( $feature )or+;
            )*
        }
    };
}
