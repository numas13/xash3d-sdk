mod save_restore;

use syn::{parse_macro_input, DeriveInput};

use crate::save_restore::SaveRestore;

#[proc_macro_derive(Save, attributes(save))]
pub fn derive_save(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let tokens = match SaveRestore::new(&input).map(|i| i.impl_save()) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    };
    tokens.into()
}

#[proc_macro_derive(Restore, attributes(save))]
pub fn derive_restore(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let tokens = match SaveRestore::new(&input).map(|i| i.impl_restore()) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    };
    tokens.into()
}
