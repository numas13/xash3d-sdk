use std::ffi::{CStr, CString};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse::Parse, spanned::Spanned, DeriveInput, Error, Token};

trait Combine<T> {
    fn combine(&mut self, other: T);
}

impl<T> Combine<Error> for Result<T, Error> {
    fn combine(&mut self, error: Error) {
        match self {
            Ok(_) => *self = Err(error),
            Err(e) => e.combine(error),
        }
    }
}

impl<T, U> Combine<Result<U, Error>> for Result<T, Error> {
    fn combine(&mut self, other: Result<U, Error>) {
        if let Err(error) = other {
            match self {
                Ok(_) => *self = Err(error),
                Err(e) => e.combine(error),
            }
        }
    }
}

fn make_cstr(s: &CStr) -> syn::LitCStr {
    syn::LitCStr::new(s, Span::call_site())
}

fn make_bstr(s: &str) -> syn::LitByteStr {
    syn::LitByteStr::new(s.as_bytes(), Span::call_site())
}

struct Field {
    member: syn::Member,
    rename: Option<String>,
    alias: Vec<String>,
    flatten: bool,
    skip_save: bool,
    skip_restore: bool,
    global: bool,
}

impl Field {
    fn new(member: syn::Member, field: &syn::Field) -> Result<Self, Error> {
        let mut result = Ok(());
        let mut ret = Self {
            member,
            rename: None,
            alias: Vec::new(),
            flatten: false,
            skip_save: false,
            skip_restore: false,
            global: false,
        };

        for attr in &field.attrs {
            result.combine(ret.parse_attr(attr));
        }

        result.map(|_| ret)
    }

    fn parse_attr(&mut self, attr: &syn::Attribute) -> Result<(), Error> {
        if !attr.path().is_ident("save") {
            return Ok(());
        }

        attr.parse_nested_meta(|meta| {
            // rename = "str"
            if meta.path.is_ident("rename") {
                meta.input.parse::<Token![=]>()?;
                let lit = meta.input.parse::<syn::LitStr>()?;
                self.rename = Some(lit.value());
                return Ok(());
            }

            // alias = "str"
            if meta.path.is_ident("alias") {
                meta.input.parse::<Token![=]>()?;
                let lit = meta.input.parse::<syn::LitStr>()?;
                self.alias.push(lit.value());
                return Ok(());
            }

            // flatten
            if meta.path.is_ident("flatten") {
                self.flatten = true;
                return Ok(());
            }

            // skip
            if meta.path.is_ident("skip") {
                self.skip_save = true;
                self.skip_restore = true;
                return Ok(());
            }

            // skip_save
            if meta.path.is_ident("skip_save") {
                self.skip_save = true;
                return Ok(());
            }

            // skip_restore
            if meta.path.is_ident("skip_restore") {
                self.skip_restore = true;
                return Ok(());
            }

            // global
            if meta.path.is_ident("global") {
                self.global = true;
                return Ok(());
            }

            Err(Error::new(meta.path.span(), "unexpected field attribute"))
        })
    }
}

struct Container {
    version: u32,
    fields: Vec<Field>,
}

impl Container {
    fn new(attrs: &[syn::Attribute], fields: &syn::Fields) -> Result<Self, Error> {
        let mut result = Ok(());
        let mut ret = Self {
            version: 0,
            fields: Vec::with_capacity(fields.len()),
        };

        for attr in attrs {
            result.combine(ret.parse_attr(attr));
        }

        for (member, field) in fields.members().zip(fields) {
            match Field::new(member, field) {
                Ok(field) => ret.fields.push(field),
                Err(error) => result.combine(error),
            }
        }

        result.map(|_| ret)
    }

    fn parse_attr(&mut self, attr: &syn::Attribute) -> Result<(), Error> {
        if !attr.path().is_ident("save") {
            return Ok(());
        }

        attr.parse_nested_meta(|meta| {
            // version = int or "str"
            if meta.path.is_ident("version") {
                meta.input.parse::<Token![=]>()?;
                if meta.input.peek(syn::LitStr) {
                    let lit = meta.input.parse::<syn::LitStr>()?;
                    self.version = lit.parse_with(syn::LitInt::parse)?.base10_parse()?;
                    return Ok(());
                }
                if meta.input.peek(syn::LitInt) {
                    let lit = meta.input.parse::<syn::LitInt>()?;
                    self.version = lit.base10_parse()?;
                    return Ok(());
                }
                return Err(Error::new(meta.input.span(), "unexpected version"));
            }

            Err(Error::new(meta.path.span(), "unexpected attribute"))
        })
    }

    fn make_save(&self) -> TokenStream {
        if self.fields.is_empty() {
            return TokenStream::new();
        }

        let mut tokens = TokenStream::new();

        for field in &self.fields {
            if field.skip_save {
                continue;
            }

            let member = &field.member;
            if field.flatten {
                tokens.extend(quote! { self.#member.save(state, cur)?; });
                continue;
            }

            let name = if let Some(name) = &field.rename {
                CString::new(name.clone()).unwrap()
            } else {
                match member {
                    syn::Member::Named(ident) => CString::new(ident.to_string()).unwrap(),
                    syn::Member::Unnamed(_) => todo!(),
                }
            };
            let name = make_cstr(&name);
            tokens.extend(quote! {
                cur.write_field(state, #name, &self.#member)?;
            });
        }

        quote! {
            #[allow(unused_imports)]
            use ::xash3d_server::save::Save;
            #tokens
            Ok(())
        }
    }

    fn make_restore_field(&self) -> TokenStream {
        let mut matches = TokenStream::new();
        let mut flatten = TokenStream::new();

        for field in &self.fields {
            if field.skip_restore {
                continue;
            }

            let member = &field.member;
            if field.flatten {
                flatten.extend(quote! {
                    if self.#member.restore_field(state, cur, name)? {
                        return Ok(true);
                    }
                });
                continue;
            }

            let mut pattern = TokenStream::new();
            if let Some(name) = &field.rename {
                let lit = make_bstr(name);
                pattern.extend(quote! { #lit });
            } else {
                let name = match member {
                    syn::Member::Named(name) => name.to_string(),
                    syn::Member::Unnamed(_index) => todo!(),
                };
                let lit = make_bstr(&name);
                pattern.extend(quote! { #lit });
            }

            for alias in &field.alias {
                let lit = make_bstr(alias);
                pattern.extend(quote! { | #lit });
            }

            if field.global {
                matches.extend(quote! {
                    #pattern => if !state.global() {
                        self.#member.restore(state, cur)?
                    },
                });
            } else {
                matches.extend(quote! {
                    #pattern => self.#member.restore(state, cur)?,
                });
            }
        }

        quote! {
            #[allow(unused_imports)]
            use ::xash3d_server::save::{Restore, RestoreField};
            match name.to_bytes() {
                #matches
                _ => {
                    #flatten
                    return Ok(false);
                }
            }
            #[allow(unreachable_code)]
            Ok(true)
        }
    }

    fn make_restore(&self) -> TokenStream {
        quote! {
            use ::xash3d_server::save::RestoreField;
            while !cur.is_empty() {
                let field = cur.read_field()?;
                let Some(name) = state.token_str(field.token()) else {
                    ::log::warn!("restore: token({}) not found", field.token().to_u16());
                    continue;
                };
                self.restore_field(state, &mut field.cursor(), name.as_c_str())?;
            }
            ::xash3d_server::entity::static_trait_cast!(
                Self, ::xash3d_server::save::OnRestore, self, mut,
            ).map(|this| this.on_restore());
            Ok(())
        }
    }
}

struct Variant {
    #[allow(dead_code)]
    container: Container,
}

impl Variant {
    fn new(variant: &syn::Variant) -> Result<Self, Error> {
        let container = Container::new(&variant.attrs, &variant.fields)?;
        Ok(Self { container })
    }
}

enum Kind {
    Struct(Container),
    Enum(Vec<Variant>),
}

pub struct SaveRestore<'a> {
    input: &'a DeriveInput,
    generics: syn::Generics,
    kind: Kind,
}

impl<'a> SaveRestore<'a> {
    pub fn new(input: &'a DeriveInput) -> Result<Self, Error> {
        let kind = match &input.data {
            syn::Data::Struct(data) => {
                let container = Container::new(&input.attrs, &data.fields)?;
                Kind::Struct(container)
            }
            syn::Data::Enum(data) => {
                let mut variants = Vec::with_capacity(data.variants.len());
                let mut result = Ok(());
                for variant in &data.variants {
                    match Variant::new(variant) {
                        Ok(variant) => variants.push(variant),
                        Err(error) => result.combine(error),
                    }
                }
                result.map(|_| Kind::Enum(variants))?
            }
            syn::Data::Union(data) => {
                return Err(Error::new_spanned(
                    data.union_token,
                    "derive SaveRestore for unions is not supported",
                ))
            }
        };

        Ok(Self {
            input,
            generics: input.generics.clone(),
            kind,
        })
    }

    pub fn impl_save(&self) -> TokenStream {
        let body = match &self.kind {
            Kind::Struct(container) => container.make_save(),
            Kind::Enum(_variants) => todo!(),
        };

        let name = &self.input.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        quote! {
            impl #impl_generics ::xash3d_server::save::Save for #name #ty_generics
            #where_clause
            {
                fn save(
                    &self,
                    state: &mut ::xash3d_server::save::SaveState,
                    cur: &mut ::xash3d_server::save::CursorMut,
                ) -> ::xash3d_server::save::SaveResult<()> {
                    #body
                }
            }
        }
    }

    pub fn impl_restore_field(&self) -> TokenStream {
        let body = match &self.kind {
            Kind::Struct(container) => container.make_restore_field(),
            Kind::Enum(_variants) => todo!(),
        };

        let name = &self.input.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        quote! {
            impl #impl_generics ::xash3d_server::save::RestoreField for #name #ty_generics
            #where_clause
            {
                fn restore_field(
                    &mut self,
                    state: &::xash3d_server::save::RestoreState,
                    cur: &mut ::xash3d_server::save::Cursor,
                    name: &::core::ffi::CStr,
                ) -> ::xash3d_server::save::SaveResult<bool> {
                    #body
                }
            }
        }
    }

    pub fn impl_restore(&self) -> TokenStream {
        let body = match &self.kind {
            Kind::Struct(container) => container.make_restore(),
            Kind::Enum(_variants) => todo!(),
        };

        let name = &self.input.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let mut tokens = quote! {
            impl #impl_generics ::xash3d_server::save::Restore for #name #ty_generics
            #where_clause
            {
                fn restore(
                    &mut self,
                    state: &::xash3d_server::save::RestoreState,
                    cur: &mut ::xash3d_server::save::Cursor,
                ) -> ::xash3d_server::save::SaveResult<()> {
                    #body
                }
            }
        };

        if let syn::Data::Struct(data) = &self.input.data {
            if let syn::Fields::Named(..) = &data.fields {
                tokens.extend(self.impl_restore_field());
            }
        }

        tokens
    }
}
