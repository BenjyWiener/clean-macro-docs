extern crate proc_macro2;

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren};
use syn::{Attribute, Error, Ident, MacroDelimiter, Path, Result, Token};

fn parse_macro_delimeter(input: ParseStream) -> Result<(MacroDelimiter, TokenStream)> {
    input.step(|cursor| {
        if let Some((TokenTree::Group(g), new_cursor)) = cursor.token_tree() {
            match g.delimiter() {
                Delimiter::Parenthesis => Ok((
                    (MacroDelimiter::Paren(Paren(g.span())), g.stream()),
                    new_cursor,
                )),
                Delimiter::Brace => Ok((
                    (MacroDelimiter::Brace(Brace(g.span())), g.stream()),
                    new_cursor,
                )),
                Delimiter::Bracket => Ok((
                    (MacroDelimiter::Bracket(Bracket(g.span())), g.stream()),
                    new_cursor,
                )),
                Delimiter::None => Err(Error::new(g.span(), "expected delimited group")),
            }
        } else {
            Err(Error::new(cursor.span(), "expected delimited group"))
        }
    })
}

macro_rules! macro_delimited {
    ($content:ident in $input:expr) => {{
        let (delim, content) = parse_macro_delimeter($input)?;
        $content = content;
        delim
    }};
}

fn macro_delimiter_surround<F>(delim: &MacroDelimiter, tokens: &mut TokenStream, f: F)
where
    F: FnOnce(&mut TokenStream),
{
    match delim {
        MacroDelimiter::Paren(paren) => paren.surround(tokens, f),
        MacroDelimiter::Brace(brace) => brace.surround(tokens, f),
        MacroDelimiter::Bracket(bracket) => bracket.surround(tokens, f),
    }
}

#[derive(Clone)]
pub struct MacroRulesRule {
    pub rule_delimiter: MacroDelimiter,
    pub rule: TokenStream,
    pub fat_arrow: Token![=>],
    pub body_delimiter: MacroDelimiter,
    pub body: TokenStream,
}

impl Parse for MacroRulesRule {
    fn parse(input: ParseStream) -> Result<Self> {
        let rule;
        let body;
        Ok(MacroRulesRule {
            rule_delimiter: macro_delimited!(rule in input),
            rule,
            fat_arrow: input.parse::<Token![=>]>()?,
            body_delimiter: macro_delimited!(body in input),
            body,
        })
    }
}

impl ToTokens for MacroRulesRule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        macro_delimiter_surround(&self.rule_delimiter, tokens, |tokens| {
            self.rule.to_tokens(tokens)
        });
        self.fat_arrow.to_tokens(tokens);
        macro_delimiter_surround(&self.body_delimiter, tokens, |tokens| {
            self.body.to_tokens(tokens)
        });
    }
}

#[derive(Clone)]
pub struct MacroRules {
    pub attrs: Vec<Attribute>,
    pub path: Path,
    pub bang_token: Token![!],
    pub ident: Ident,
    pub delimiter: MacroDelimiter,
    pub rules: Punctuated<MacroRulesRule, Token![;]>,
    pub semi_token: Option<Token![;]>,
}

impl Parse for MacroRules {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(MacroRules {
            attrs: input.call(Attribute::parse_outer)?,
            path: input.call(Path::parse_mod_style)?,
            bang_token: input.parse()?,
            ident: input.parse()?,
            delimiter: macro_delimited!(content in input),
            rules: Punctuated::<MacroRulesRule, Token![;]>::parse_terminated.parse2(content)?,
            semi_token: input.parse().ok(),
        })
    }
}

impl ToTokens for MacroRules {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.path.to_tokens(tokens);
        self.bang_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        macro_delimiter_surround(&self.delimiter, tokens, |tokens| {
            self.rules.to_tokens(tokens)
        });
        if let Some(semi) = self.semi_token {
            semi.to_tokens(tokens);
        }
    }
}
