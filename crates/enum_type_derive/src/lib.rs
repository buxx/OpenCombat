// /!\ AI GENERATED /!\
//
//! `#[derive(EnumType)]`
//!
//! Given:
//! ```ignore
//! #[derive(EnumType)]
//! pub enum Order {
//!     Idle,
//!     MoveTo(Position),
//!     Attack { target: Entity, power: u32 },
//! }
//! ```
//!
//! generates a sibling "tag" enum whose variants have the same names but
//! no payload at all:
//!
//! ```ignore
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! pub enum OrderType {
//!     Idle,
//!     MoveTo,
//!     Attack,
//! }
//!
//! impl Order {
//!     pub fn order_type(&self) -> OrderType { ... }
//! }
//!
//! impl From<&Order> for OrderType { ... }
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Fields, Ident};

#[proc_macro_derive(EnumType, attributes(enum_type))]
pub fn derive_enum_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match expand(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    };

    TokenStream::from(expanded)
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let enum_name = &input.ident;
    let vis = &input.vis;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let data_enum = match &input.data {
        Data::Enum(data_enum) => data_enum,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "EnumType can only be derived for enums",
            ))
        }
    };

    // Optional override: #[enum_type(name = "Foo", derive(Debug, Serialize))]
    let mut type_name = Ident::new(&format!("{}Type", enum_name), enum_name.span());
    let mut extra_derives: Vec<syn::Path> = Vec::new();

    for attr in &input.attrs {
        if attr.path().is_ident("enum_type") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let s: syn::LitStr = value.parse()?;
                    type_name = Ident::new(&s.value(), s.span());
                    Ok(())
                } else if meta.path.is_ident("derive") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let paths: Punctuated<syn::Path, syn::Token![,]> =
                        Punctuated::parse_terminated(&content)?;
                    extra_derives.extend(paths);
                    Ok(())
                } else {
                    Err(meta.error("unsupported enum_type attribute"))
                }
            })?;
        }
    }

    let variant_idents: Vec<&Ident> = data_enum.variants.iter().map(|v| &v.ident).collect();

    if variant_idents.is_empty() {
        return Err(syn::Error::new_spanned(
            enum_name,
            "EnumType requires at least one variant",
        ));
    }

    let match_arms = data_enum.variants.iter().map(|v| {
        let vident = &v.ident;
        let pattern = match &v.fields {
            Fields::Unit => quote! { #enum_name::#vident },
            Fields::Unnamed(_) => quote! { #enum_name::#vident(..) },
            Fields::Named(_) => quote! { #enum_name::#vident { .. } },
        };
        quote! { #pattern => #type_name::#vident }
    });

    let method_name = Ident::new(&to_snake_case(&type_name.to_string()), Span::call_site());

    let base_derives = quote! { #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] };
    let extra_derive_attr = if extra_derives.is_empty() {
        quote! {}
    } else {
        quote! { #[derive(#(#extra_derives),*)] }
    };

    let expanded = quote! {
        #base_derives
        #extra_derive_attr
        #vis enum #type_name {
            #(#variant_idents),*
        }

        impl #impl_generics #enum_name #ty_generics #where_clause {
            /// Returns the fieldless "kind" of this value.
            #vis fn #method_name(&self) -> #type_name {
                match self {
                    #(#match_arms),*
                }
            }
        }

        impl #impl_generics ::core::convert::From<&#enum_name #ty_generics> for #type_name #where_clause {
            fn from(value: &#enum_name #ty_generics) -> Self {
                value.#method_name()
            }
        }
    };

    Ok(expanded)
}

/// Minimal PascalCase -> snake_case converter (no external dependency needed).
fn to_snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    let mut prev_lower_or_digit = false;
    for c in s.chars() {
        if c.is_uppercase() {
            if prev_lower_or_digit {
                out.push('_');
            }
            out.extend(c.to_lowercase());
            prev_lower_or_digit = false;
        } else {
            out.push(c);
            prev_lower_or_digit = c.is_lowercase() || c.is_numeric();
        }
    }
    out
}
