use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Item, parse_macro_input};

/// Make a struct or enum archivable with rkyv (and optionally serializable with serde).
///
/// ```ignore
/// #[oc_derive::archivable(serde)]
/// #[derive(Debug, Clone, PartialEq)]
/// pub struct Foo { .. }
/// ```
///
/// expands to:
///
/// ```ignore
/// #[derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
/// #[derive(::serde::Serialize, ::serde::Deserialize)]
/// #[rkyv(compare(PartialEq), derive(Debug))]
/// #[derive(Debug, Clone, PartialEq)]
/// pub struct Foo { .. }
/// ```
///
/// Options (comma separated):
/// - `serde`: also derive `serde::Serialize` and `serde::Deserialize`.
/// - `no_compare`: archived type only derives `Debug` (`#[rkyv(derive(Debug))]`).
/// - `bare`: no `#[rkyv(..)]` attribute at all.
///
/// Must be placed **before** `#[derive(..)]` so that rkyv/serde helper attributes
/// (`#[rkyv(..)]`, `#[serde(..)]`) on fields are recognized.
#[proc_macro_attribute]
pub fn archivable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    match expand_archivable(attr.into(), item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_archivable(attr: TokenStream2, item: Item) -> syn::Result<TokenStream2> {
    if !matches!(item, Item::Struct(_) | Item::Enum(_)) {
        return Err(syn::Error::new_spanned(
            item,
            "archivable can only be used on structs and enums",
        ));
    }

    let mut serde = false;
    let mut compare = true;
    let mut rkyv_attr = true;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("serde") {
            serde = true;
            Ok(())
        } else if meta.path.is_ident("no_compare") {
            compare = false;
            Ok(())
        } else if meta.path.is_ident("bare") {
            rkyv_attr = false;
            Ok(())
        } else {
            Err(meta.error("unsupported archivable option (expected serde, no_compare or bare)"))
        }
    });
    syn::parse::Parser::parse2(parser, attr)?;

    let serde_derive =
        serde.then(|| quote! { #[derive(::serde::Serialize, ::serde::Deserialize)] });
    let rkyv_attr = match (rkyv_attr, compare) {
        (false, _) => quote! {},
        (true, true) => quote! { #[rkyv(compare(PartialEq), derive(Debug))] },
        (true, false) => quote! { #[rkyv(derive(Debug))] },
    };

    Ok(quote! {
        #[derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
        #serde_derive
        #rkyv_attr
        #item
    })
}
