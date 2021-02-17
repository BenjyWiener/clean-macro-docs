extern crate proc_macro2;

use if_chain::if_chain;
use proc_macro2::{Delimiter, Punct, Spacing, Span, TokenStream, TokenTree};
use std::iter::FromIterator;
use syn::Ident;

// Replace all occurences of `pub_ident!(priv_marker ...)` (using any delimiter)
// with `priv_ident!(priv_marker ...)`.
pub fn replace_macro_invocs(
    stream: TokenStream,
    pub_ident: &Ident,
    priv_ident: &Ident,
    priv_marker: &TokenStream,
) -> TokenStream {
    let mut tokens: Vec<TokenTree> = stream.into_iter().collect();

    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Group(group) = &tokens[i] {
            tokens[i] = TokenTree::Group(proc_macro2::Group::new(
                group.delimiter(),
                replace_macro_invocs(group.stream(), pub_ident, priv_ident, priv_marker),
            ));
        } else if let TokenTree::Ident(ident) = &tokens[i] {
            if_chain! {
                if tokens.len() - i >= 3 && ident == pub_ident;
                if let TokenTree::Punct(punct) = &tokens[i + 1];
                if punct.as_char() == '!';
                // pub_ident! ...
                if let TokenTree::Group(group) = &tokens[i + 2];
                if group.delimiter() != Delimiter::None;
                // pub_ident!( ... )
                if group.stream().to_string().starts_with(&priv_marker.to_string());
                then {
                    tokens.splice(i..=i, vec![
                        // Use Spacing::Alone for the `$` to make string-based
                        // tests work correctly.
                        TokenTree::Punct(Punct::new('$', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("crate", Span::call_site())),
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Ident(priv_ident.clone()),
                    ]);
                    i += 4;
                }
            }
        }
        i += 1;
    }
    TokenStream::from_iter(tokens)
}
