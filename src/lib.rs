//! Hide internal rules when documenting `macro_rules!` macros.
//!
//! When generating docs for `macro_rules!` macros, `rustdoc` will include every
//! rule, including internal rules that are only supposed to be called from within
//! your macro. The `clean_docs` attribute will hide your internal rules from
//! `rustdoc`.
//!
//! # Example:
//! ```
//! # use clean_macro_docs::clean_docs;
//! #[macro_export]
//! macro_rules! messy {
//!     (@impl $e:expr) => {
//!         format!("{}", $e)
//!     };
//!     ($e:expr) => {
//!         messy!(@impl $e)
//!     };
//! }
//!
//! #[clean_docs]
//! #[macro_export]
//! macro_rules! clean {
//!     (@impl $e:expr) => {
//!         format!("{}", $e)
//!     };
//!     ($e:expr) => {
//!         clean!(@impl $e)
//!     };
//! }
//! ```
//!
//! would be documented as
//! ```
//! macro_rules! mac {
//!     ($e:expr) => { ... };
//! }
//! ```
//! # How does it work?
//! The macro above is transformed into
//! ```
//! #[macro_export]
//! macro_rules! clean {
//!     ($e:expr) => {
//!         $crate::__clean!(@impl $e)
//!     };
//! }
//!
//! #[macro_export]
//! macro_rules! __clean {
//!     (@impl $e:expr) => {
//!         format!("{}", $e)
//!     };
//! }
//!
//! macro_rules! clean {
//!     (@impl $e:expr) => {
//!         format!("{}", $e)
//!     };
//!     ($e:expr) => {
//!         clean!(@impl $e)
//!     };
//! }
//! ```
//!
//! The last, non-`macro_export` macro is there becuase Rust doesn't allow
//! macro-expanded macro to be invoked by absolute path (i.e. `$crate::__mac`).
//!
//! The solution is to shadow the `macro_export`ed macro with a local version
//! that doesn't use absolute paths.
//!
//! By default this transformation only happens when `rustdoc` is building the
//! documentation for your macro, so `clean_docs` shouldn't affect your normal
//! compilation times (see [`always`](#always)).
//!
//! # Arguments
//! You can use these optional arguments to configure `clean_macro`.
//!
//! ```
//! # use clean_macro_docs::clean_docs;
//! #[clean_docs(impl = "#internal", internal = "__internal_mac", always = true)]
//! # macro_rules! mac { () => {} }
//! ```
//!
//! ## `impl`
//! A string representing the "flag" at the begining of an internal rule. Defaults to `"@"`.
//!
//! ```
//! # use clean_macro_docs::clean_docs;
//! #[clean_docs(impl = "#internal")]
//! #[macro_export]
//! macro_rules! mac {
//!     (#internal $e:expr) => {
//!         format!("{}", $e)
//!     };
//!     ($e:expr) => {
//!         mac!(#internal $e)
//!     };
//! }
//! ```
//!
//! ## `internal`
//! A string representing the identifier to use for the internal version of your macro.
//! By default `clean_docs` prepends `__` (two underscores) to the main macro's identifier.
//!
//! ```
//! # use clean_macro_docs::clean_docs;
//! #[clean_docs(internal = "__internal_mac")]
//! #[macro_export]
//! macro_rules! mac {
//!     (@impl $e:expr) => {
//!         format!("{}", $e)
//!     };
//!     ($e:expr) => {
//!         mac!(@impl $e)
//!     };
//! }
//! ```
//!
//! ## `always`
//! A boolean that tells `clean_docs` whether it should transform the macro
//! even when not building documentation. This is mainly used for testing
//! purposes. Defaults to `false`.
//!
//! ```
//! # use clean_macro_docs::clean_docs;
//! #[clean_docs(always = true)]
//! #[macro_export]
//! macro_rules! mac {
//!     (@impl $e:expr) => {
//!         format!("{}", $e)
//!     };
//!     ($e:expr) => {
//!         mac!(@impl $e)
//!     };
//! }
//! ```

extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::{Punct, Spacing, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use std::str::FromStr;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, Ident, Lit, Meta, NestedMeta, Token};

mod macro_rules;
mod replace_macro_invocs;

use macro_rules::*;
use replace_macro_invocs::replace_macro_invocs;

#[proc_macro_attribute]
pub fn clean_docs(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let mac_rules = parse_macro_input!(item as MacroRules);
    clean_docs_impl(args, mac_rules).into()
}

fn clean_docs_impl(args: AttributeArgs, mut mac_rules: MacroRules) -> TokenStream {
    let mut run_always = false;
    let mut priv_marker: Option<TokenStream> = None;
    let mut priv_ident: Option<Ident> = None;

    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(arg)) = arg {
            match (
                arg.path
                    .get_ident()
                    .map(Ident::to_string)
                    .as_ref()
                    .map(String::as_str),
                arg.lit,
            ) {
                (Some("always"), Lit::Bool(val)) => run_always = val.value,
                (Some("impl"), Lit::Str(val)) => {
                    priv_marker = Some(TokenStream::from_str(&val.value()).expect("Invalid tokens"))
                }
                (Some("internal"), Lit::Str(val)) => {
                    priv_ident = Some(val.parse().expect("Expected valid identifier"))
                }
                _ => {
                    let path = &arg.path;
                    let path = quote!(#path).to_string();
                    return quote_spanned! {
                        arg.path.span()=> compile_error!(concat!("invalid argument: ", #path));
                    };
                }
            };
        }
    }

    // Only run when generating docs, or if always is true
    if !run_always && std::env::var("doc").is_err() {
        return quote! {
            #mac_rules
        };
    }

    // Clone item, to be reimitted unmodified without #[macro_export]
    let mut original = mac_rules.clone();

    let pub_ident = &mac_rules.ident;

    // Default values
    let priv_marker = priv_marker
        .unwrap_or_else(|| TokenStream::from(TokenTree::Punct(Punct::new('@', Spacing::Joint))));
    let priv_ident = priv_ident
        .unwrap_or_else(|| format_ident!("__{}", pub_ident));

    let mut pub_rules = Punctuated::<MacroRulesRule, Token![;]>::new();
    let mut priv_rules = Punctuated::<MacroRulesRule, Token![;]>::new();

    for mut rule in mac_rules.rules {
        rule.body = replace_macro_invocs(rule.body, pub_ident, &priv_ident, &priv_marker);
        if rule.rule.to_string().starts_with(&priv_marker.to_string()) {
            priv_rules.push(rule);
        } else {
            pub_rules.push(rule);
        }
    }

    if priv_rules.is_empty() {
        return quote! {
            #original
        };
    }

    mac_rules.rules = pub_rules;

    let mut priv_mac_rules = MacroRules {
        ident: priv_ident,
        rules: priv_rules,
        ..mac_rules.clone()
    };

    // Remove doc comments (and other doc attrs) from private version
    priv_mac_rules.attrs.retain(|attr| {
        if let Some(ident) = attr.path.get_ident() {
            ident.to_string() != "doc"
        } else {
            true
        }
    });

    // Remove #[macro_export] and doc comments (and other doc attrs) from crate-internal version
    original.attrs.retain(|attr| {
        if let Some(ident) = attr.path.get_ident() {
            ident.to_string() != "macro_export" && ident.to_string() != "doc"
        } else {
            true
        }
    });

    let gen = quote! {
        #mac_rules
        #[doc(hidden)]
        #priv_mac_rules

        #[allow(unused_macros)]
        #original
    };
    gen.into()
}

#[cfg(test)]
mod tests;
