mod save_restore;

use syn::{DeriveInput, parse_macro_input};

use crate::save_restore::SaveRestore;

#[proc_macro_derive(Save, attributes(save))]
pub fn derive_save(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let tokens = match SaveRestore::new(&input) {
        Ok(save_restore) => save_restore.impl_save_trait(),
        Err(err) => err.to_compile_error(),
    };
    tokens.into()
}

#[proc_macro_derive(Restore, attributes(save))]
pub fn derive_restore(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let tokens = match SaveRestore::new(&input) {
        Ok(save_restore) => save_restore.impl_restore_trait(),
        Err(err) => err.to_compile_error(),
    };
    tokens.into()
}
