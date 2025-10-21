use std::ffi::{CStr, CString};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, spanned::Spanned, DeriveInput, Error, Token};

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

#[derive(Default)]
struct StructAttrs {}

impl StructAttrs {
    fn parse(attrs: &[syn::Attribute]) -> Result<Self, Error> {
        let ret = Self::default();
        let mut result = Ok(());
        for attr in attrs.iter() {
            if !attr.path().is_ident("save") {
                continue;
            }
            let parse_result = attr.parse_nested_meta(|meta| {
                // no attributes yet
                Err(Error::new(meta.path.span(), "unexpected attribute"))
            });
            result.combine(parse_result);
        }
        result.map(|_| ret)
    }
}

#[derive(Default)]
struct EnumAttrs {
    repr: Option<String>,
}

impl EnumAttrs {
    fn parse(attrs: &[syn::Attribute]) -> Result<Self, Error> {
        let mut ret = Self::default();
        let mut result = Ok(());
        for attr in attrs.iter() {
            if attr.path().is_ident("repr") {
                let list = match attr.meta.require_list() {
                    Ok(list) => list,
                    Err(error) => {
                        result.combine(error);
                        continue;
                    }
                };
                match syn::parse2::<syn::Ident>(list.tokens.clone()) {
                    Ok(i) => ret.repr = Some(i.to_string()),
                    Err(error) => {
                        result.combine(error);
                        continue;
                    }
                }
                continue;
            }

            if !attr.path().is_ident("save") {
                continue;
            }

            let parse_result = attr.parse_nested_meta(|meta| {
                // no attributes yet
                Err(Error::new(meta.path.span(), "unexpected attribute"))
            });
            result.combine(parse_result);
        }
        result.map(|_| ret)
    }
}

#[derive(Default)]
struct VariantAttrs {}

impl VariantAttrs {
    fn parse(attrs: &[syn::Attribute]) -> Result<Self, Error> {
        let ret = Self::default();
        let mut result = Ok(());
        for attr in attrs.iter() {
            if !attr.path().is_ident("save") {
                continue;
            }
            let parse_result = attr.parse_nested_meta(|meta| {
                // no attributes yet
                Err(Error::new(meta.path.span(), "unexpected attribute"))
            });
            result.combine(parse_result);
        }
        result.map(|_| ret)
    }
}

#[derive(Default)]
struct FieldAttrs {
    rename: Option<String>,
    alias: Vec<String>,
    flatten: bool,
    skip_save: bool,
    skip_restore: bool,
    global: bool,
}

impl FieldAttrs {
    fn parse(attrs: &[syn::Attribute]) -> Result<Self, Error> {
        let mut ret = Self::default();
        let mut result = Ok(());
        for attr in attrs.iter().filter(|i| i.path().is_ident("save")) {
            let parse_result = attr.parse_nested_meta(|meta| {
                // rename = "str"
                if meta.path.is_ident("rename") {
                    meta.input.parse::<Token![=]>()?;
                    let lit = meta.input.parse::<syn::LitStr>()?;
                    ret.rename = Some(lit.value());
                    return Ok(());
                }

                // alias = "str"
                if meta.path.is_ident("alias") {
                    meta.input.parse::<Token![=]>()?;
                    let lit = meta.input.parse::<syn::LitStr>()?;
                    ret.alias.push(lit.value());
                    return Ok(());
                }

                // flatten
                if meta.path.is_ident("flatten") {
                    ret.flatten = true;
                    return Ok(());
                }

                // skip
                if meta.path.is_ident("skip") {
                    ret.skip_save = true;
                    ret.skip_restore = true;
                    return Ok(());
                }

                // skip_save
                if meta.path.is_ident("skip_save") {
                    ret.skip_save = true;
                    return Ok(());
                }

                // skip_restore
                if meta.path.is_ident("skip_restore") {
                    ret.skip_restore = true;
                    return Ok(());
                }

                // global
                if meta.path.is_ident("global") {
                    ret.global = true;
                    return Ok(());
                }

                Err(Error::new(meta.path.span(), "unexpected attribute"))
            });
            result.combine(parse_result);
        }
        result.map(|_| ret)
    }
}

fn parse_fields_attrs(fields: &syn::Fields) -> Result<Vec<FieldAttrs>, Error> {
    let mut result = Ok(());
    let mut ret = Vec::new();
    for field in fields.iter() {
        match FieldAttrs::parse(&field.attrs) {
            Ok(attrs) => ret.push(attrs),
            Err(error) => result.combine(error),
        }
    }
    result.map(|_| ret)
}

pub struct SaveRestore<'a> {
    input: &'a DeriveInput,
    #[allow(dead_code)]
    struct_attrs: StructAttrs,
    enum_attrs: EnumAttrs,
    #[allow(dead_code)]
    variant_attrs: Vec<VariantAttrs>,
    field_attrs: Vec<Vec<FieldAttrs>>,
}

impl<'a> SaveRestore<'a> {
    pub fn new(input: &'a DeriveInput) -> Result<Self, Error> {
        let mut result = Ok(());
        let mut struct_attrs = StructAttrs::default();
        let mut enum_attrs = EnumAttrs::default();
        let mut variant_attrs = Vec::new();
        let mut field_attrs = Vec::new();
        match &input.data {
            syn::Data::Struct(data) => {
                match StructAttrs::parse(&input.attrs) {
                    Ok(attrs) => struct_attrs = attrs,
                    Err(error) => result.combine(error),
                }
                match parse_fields_attrs(&data.fields) {
                    Ok(attrs) => field_attrs.push(attrs),
                    Err(error) => result.combine(error),
                }
            }
            syn::Data::Enum(data) => {
                match EnumAttrs::parse(&input.attrs) {
                    Ok(attrs) => enum_attrs = attrs,
                    Err(error) => result.combine(error),
                }
                for variant in &data.variants {
                    match VariantAttrs::parse(&variant.attrs) {
                        Ok(attrs) => variant_attrs.push(attrs),
                        Err(error) => result.combine(error),
                    }
                    match parse_fields_attrs(&variant.fields) {
                        Ok(attrs) => field_attrs.push(attrs),
                        Err(error) => result.combine(error),
                    }
                }
            }
            syn::Data::Union(data) => {
                let err = "derive(Save) for union is not supported";
                return Err(Error::new(data.union_token.span(), err));
            }
        }

        result.map(|_| Self {
            input,
            struct_attrs,
            enum_attrs,
            variant_attrs,
            field_attrs,
        })
    }

    fn add_where_predicates_fields(
        &self,
        generics: &mut syn::Generics,
        bound: syn::TypeParamBound,
        skip: impl Fn(&FieldAttrs) -> bool,
        fields: &syn::Fields,
        attrs: &[FieldAttrs],
    ) {
        for param in self.input.generics.type_params() {
            for (field, attrs) in fields.iter().zip(attrs) {
                if skip(attrs) {
                    continue;
                }
                if let syn::Type::Path(ty) = &field.ty {
                    if ty.path.get_ident() == Some(&param.ident) {
                        add_where_predicate(generics, &param.ident, bound.clone());
                        break;
                    }
                }
            }
        }
    }

    fn add_where_predicates(
        &self,
        generics: &mut syn::Generics,
        bound: syn::TypeParamBound,
        skip: impl Fn(&FieldAttrs) -> bool + Copy,
    ) {
        match &self.input.data {
            syn::Data::Struct(data) => {
                self.add_where_predicates_fields(
                    generics,
                    bound,
                    skip,
                    &data.fields,
                    &self.field_attrs[0],
                );
            }
            syn::Data::Enum(data) => {
                for (variant, attrs) in data.variants.iter().zip(&self.field_attrs) {
                    self.add_where_predicates_fields(
                        generics,
                        bound.clone(),
                        skip,
                        &variant.fields,
                        attrs,
                    );
                }
            }
            syn::Data::Union(..) => unreachable!("union"),
        }
    }

    fn add_where_predicates_for_save(&self, generics: &mut syn::Generics) {
        let path = parse_quote!(::xash3d_server::save::Save);
        let bound = make_type_param_trait_bound(path);
        self.add_where_predicates(generics, bound, |attrs| attrs.skip_save);
    }

    fn add_where_predicates_for_restore(&self, generics: &mut syn::Generics) {
        let path = parse_quote!(::xash3d_server::save::Restore);
        let bound = make_type_param_trait_bound(path);
        self.add_where_predicates(generics, bound, |attrs| attrs.skip_restore || attrs.flatten);

        let path = parse_quote!(::xash3d_server::save::RestoreField);
        let bound = make_type_param_trait_bound(path);
        self.add_where_predicates(generics, bound, |attrs| {
            attrs.skip_restore || !attrs.flatten
        });

        if let syn::Data::Enum(..) = &self.input.data {
            let path = parse_quote!(::xash3d_server::save::RestoreWithDefault);
            let bound = make_type_param_trait_bound(path);
            self.add_where_predicates(generics, bound, |attrs| attrs.skip_restore);
        }
    }

    fn default_fields(&self, fields: &syn::Fields) -> TokenStream {
        let mut tokens = TokenStream::new();
        for (i, field) in fields.iter().enumerate() {
            let name = format_ident!("f{i}");
            let ty = &field.ty;
            tokens.extend(quote! {
                let mut #name: #ty = RestoreWithDefault::default_for_restore(state);
            });
        }
        tokens
    }

    fn enumerate_fields(&self, fields: &syn::Fields) -> TokenStream {
        let mut tokens = TokenStream::new();
        for (i, member) in fields.members().enumerate() {
            let value = format_ident!("f{i}");
            match member {
                syn::Member::Named(ident) => {
                    tokens.extend(quote!(#ident: #value,));
                }
                syn::Member::Unnamed(_) => {
                    tokens.extend(quote!(#value,));
                }
            }
        }
        match fields {
            syn::Fields::Named(..) => quote!({#tokens}),
            syn::Fields::Unnamed(..) => quote!((#tokens)),
            syn::Fields::Unit => quote!(),
        }
    }

    fn save_fields(&self, fields: &syn::Fields, attrs: &[FieldAttrs]) -> TokenStream {
        let mut tokens = TokenStream::new();
        for (i, (field, attrs)) in fields.iter().zip(attrs).enumerate() {
            if attrs.skip_save {
                continue;
            }
            let member = format_ident!("f{i}");
            if attrs.flatten {
                let save_field = quote! { #member.save(state, cur)?; };
                tokens.extend(save_field);
                continue;
            }
            let name = if let Some(name) = &attrs.rename {
                name.clone()
            } else if let Some(name) = &field.ident {
                name.to_string()
            } else {
                format!("{i}")
            };
            let name = make_cstr(&CString::new(name).unwrap());
            let write_field = quote! { cur.write_field(state, #name, #member)?; };
            tokens.extend(write_field);
        }
        tokens
    }

    fn save_variants(
        &self,
        variants: impl Iterator<Item = &'a syn::Variant>,
        repr: &str,
    ) -> TokenStream {
        let write_fn = format_ident!("write_leb_{repr}");
        let mut tokens = TokenStream::new();
        let mut offset = 0_u32;
        let mut base = None;
        for (variant, fields_attrs) in variants.zip(&self.field_attrs) {
            if let Some(new_base) = &variant.discriminant {
                base = Some(&new_base.1);
                offset = 0;
            }
            let i = syn::LitInt::new(&format!("{offset}"), Span::call_site());
            let discriminant = match base {
                Some(base) if offset == 0 => quote!(#base),
                Some(base) => quote!(#base + #i),
                None => quote!(#i),
            };
            let name = &variant.ident;
            let extract_fields = self.enumerate_fields(&variant.fields);
            let save_fields = self.save_fields(&variant.fields, fields_attrs);
            tokens.extend(quote! {
                Self::#name #extract_fields => {
                    cur.#write_fn(#discriminant)?;
                    #save_fields
                },
            });
            offset += 1;
        }
        tokens
    }

    fn save_trait_body(&self) -> TokenStream {
        match &self.input.data {
            syn::Data::Struct(data) => {
                let extract_fields = self.enumerate_fields(&data.fields);
                let save_fields = self.save_fields(&data.fields, &self.field_attrs[0]);
                quote! {
                    let Self #extract_fields = self;
                    #save_fields
                }
            }
            syn::Data::Enum(data) => {
                let repr = self.enum_attrs.repr.as_deref().unwrap_or("isize");
                let save_variants = self.save_variants(data.variants.iter(), repr);
                quote! {
                    match self {
                        #save_variants
                    }
                }
            }
            syn::Data::Union(..) => unreachable!("union"),
        }
    }

    pub fn impl_save_trait(&self) -> TokenStream {
        let body = self.save_trait_body();
        let mut generics = self.input.generics.clone();
        self.add_where_predicates_for_save(&mut generics);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let name = &self.input.ident;
        quote! {
            impl #impl_generics ::xash3d_server::save::Save for #name #ty_generics
            #where_clause
            {
                fn save(
                    &self,
                    state: &mut ::xash3d_server::save::SaveState,
                    cur: &mut ::xash3d_server::save::CursorMut,
                ) -> ::xash3d_server::save::SaveResult<()> {
                    #[allow(unused_imports)]
                    use ::xash3d_server::save::Save;
                    #body
                    Ok(())
                }
            }
        }
    }

    fn restore_fields(&self, fields: &syn::Fields, attrs: &[FieldAttrs]) -> TokenStream {
        let mut tokens = TokenStream::new();

        for (i, (field, attrs)) in fields.iter().zip(attrs).enumerate() {
            if attrs.skip_restore {
                continue;
            }

            let member = format_ident!("f{i}");
            if attrs.flatten {
                tokens.extend(quote! {
                    if #member.restore_field(state, cur, name)? {
                        return Ok(true);
                    }
                });
                continue;
            }

            let mut cond = TokenStream::new();
            let name = if let Some(name) = &attrs.rename {
                name.clone()
            } else if let Some(name) = &field.ident {
                name.to_string()
            } else {
                format!("{i}")
            };
            let lit = make_cstr(&CString::new(name).unwrap());
            cond.extend(quote! { name == #lit });

            for alias in attrs.alias.iter().cloned() {
                let lit = make_cstr(&CString::new(alias).unwrap());
                cond.extend(quote! { || name == #lit });
            }

            if attrs.global {
                tokens.extend(quote! {
                    if #cond {
                        if !state.global() {
                            #member.restore(state, cur)?;
                        }
                        return Ok(true);
                    }
                });
            } else {
                tokens.extend(quote! {
                    if #cond {
                        #member.restore(state, cur)?;
                        return Ok(true);
                    }
                });
            }
        }

        tokens
    }

    fn restore_field_trait_body(&self) -> TokenStream {
        match &self.input.data {
            syn::Data::Struct(data) => {
                let extract_fields = self.enumerate_fields(&data.fields);
                let restore_fields = self.restore_fields(&data.fields, &self.field_attrs[0]);
                quote! {
                    #[allow(unused_imports)]
                    use ::xash3d_server::save::{Restore, RestoreField};
                    let Self #extract_fields = self;
                    #restore_fields
                    Ok(false)
                }
            }
            syn::Data::Enum(_data) => {
                quote! { todo!(); }
            }
            syn::Data::Union(..) => unreachable!("union"),
        }
    }

    pub fn impl_restore_field_trait(&self, generics: &syn::Generics) -> TokenStream {
        let body = self.restore_field_trait_body();
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let name = &self.input.ident;
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

    fn restore_struct(&self) -> TokenStream {
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

    fn restore_variants(&self, variants: impl Iterator<Item = &'a syn::Variant>) -> TokenStream {
        let mut tokens = TokenStream::new();
        let mut offset = 0_u32;
        let mut base = None;
        for (variant, attrs) in variants.zip(&self.field_attrs) {
            let name = &variant.ident;

            if let Some(new_base) = &variant.discriminant {
                base = Some(&new_base.1);
                offset = 0;
            }
            let i = syn::LitInt::new(&format!("{offset}"), Span::call_site());
            let discriminant = match base {
                Some(base) if offset == 0 => quote!(#base),
                Some(base) => quote!(#base + #i),
                None => quote!(#i),
            };
            offset += 1;

            if let syn::Fields::Unit = &variant.fields {
                tokens.extend(quote! {
                    if discriminant == #discriminant {
                        *self = Self::#name;
                        return Ok(());
                    }
                });
                continue;
            }

            let default_fields = self.default_fields(&variant.fields);
            let restore_field = self.restore_fields(&variant.fields, attrs);
            let collect_fields = self.enumerate_fields(&variant.fields);
            tokens.extend(quote! {
                if discriminant == #discriminant {
                    #default_fields

                    while !cur.is_empty() {
                        let field = cur.read_field()?;
                        let Some(name) = state.token_str(field.token()) else {
                            ::log::warn!("restore: token({}) not found", field.token().to_u16());
                            continue;
                        };
                        let mut restore_field = |state, cur, name| {
                            #restore_field
                            Ok(false)
                        };
                        restore_field(state, &mut field.cursor(), name.as_c_str())?;
                    }

                    *self = Self::#name #collect_fields;
                    return Ok(());
                }
            });
        }
        tokens
    }

    fn restore_enum(&self, data: &syn::DataEnum) -> TokenStream {
        let repr = self.enum_attrs.repr.as_deref().unwrap_or("isize");
        let read_fn = format_ident!("read_leb_{repr}");
        let restore_variants = self.restore_variants(data.variants.iter());
        quote! {
            #[allow(unused_imports)]
            use ::xash3d_server::save::{Restore, RestoreField, RestoreWithDefault};
            let discriminant = cur.#read_fn()?;
            #restore_variants
            Err(::xash3d_server::save::SaveError::InvalidEnum)
        }
    }

    fn restore_trait_body(&self) -> TokenStream {
        match &self.input.data {
            syn::Data::Struct(..) => self.restore_struct(),
            syn::Data::Enum(data) => self.restore_enum(data),
            syn::Data::Union(..) => unreachable!("union"),
        }
    }

    pub fn impl_restore_trait(&self) -> TokenStream {
        let body = self.restore_trait_body();
        let mut generics = self.input.generics.clone();
        self.add_where_predicates_for_restore(&mut generics);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let name = &self.input.ident;
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

        if let syn::Data::Struct(..) = &self.input.data {
            tokens.extend(self.impl_restore_field_trait(&generics));
        }

        tokens
    }
}

fn make_type_param_trait_bound(path: syn::Path) -> syn::TypeParamBound {
    syn::TypeParamBound::Trait(syn::TraitBound {
        paren_token: None,
        modifier: syn::TraitBoundModifier::None,
        lifetimes: None,
        path,
    })
}

fn make_where_predicate_type(
    bounded_ty: syn::Type,
    bound: syn::TypeParamBound,
) -> syn::WherePredicate {
    syn::WherePredicate::Type(syn::PredicateType {
        lifetimes: None,
        bounded_ty,
        colon_token: Token![:](Span::call_site()),
        bounds: syn::punctuated::Punctuated::from_iter([bound.clone()]),
    })
}

fn add_where_predicate(generics: &mut syn::Generics, ty: &syn::Ident, bound: syn::TypeParamBound) {
    let ty = syn::Type::Verbatim(ty.to_token_stream());
    generics
        .make_where_clause()
        .predicates
        .push(make_where_predicate_type(ty, bound));
}
