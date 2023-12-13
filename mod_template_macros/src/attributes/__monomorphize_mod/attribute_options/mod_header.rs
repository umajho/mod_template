use proc_macro2::TokenStream;
use syn::ext::IdentExt;

/// Taken partially from [syn::ItemMod].
///
/// LICENSE: <https://github.com/dtolnay/syn/blob/master/LICENSE-MIT>.
pub struct ModHeader {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub unsafety: Option<syn::Token![unsafe]>,
    pub mod_token: syn::Token![mod],
    pub ident: syn::Ident,
}

impl syn::parse::Parse for ModHeader {
    /// Taken partially from syn. (`impl Parse for ItemMod`)
    ///
    /// LICENSE: <https://github.com/dtolnay/syn/blob/master/LICENSE-MIT>.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(syn::Attribute::parse_outer)?,
            vis: input.parse()?,
            unsafety: input.parse()?,
            mod_token: input.parse()?,
            ident: if input.peek(syn::Token![try]) {
                input.call(syn::Ident::parse_any)?
            } else {
                input.parse()?
            },
        })
    }
}

impl quote::ToTokens for ModHeader {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            attrs,
            vis,
            unsafety,
            mod_token,
            ident,
        } = self;

        for attr in attrs {
            tokens.extend(attr.to_token_stream())
        }
        tokens.extend(vis.to_token_stream());
        tokens.extend(unsafety.to_token_stream());
        tokens.extend(mod_token.to_token_stream());
        tokens.extend(ident.to_token_stream());
    }
}
