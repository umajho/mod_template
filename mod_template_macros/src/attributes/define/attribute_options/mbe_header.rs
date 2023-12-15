pub struct MbeHeader {
    attributes: Vec<syn::Attribute>,
    vis: syn::Visibility,
    macro_keyword: MacroKeyword,
    name_ident: syn::Ident,
}

enum MacroKeyword {
    MacroRules(syn::Ident, syn::Token![!]),
    Macro(syn::Token![macro]),
}

impl MbeHeader {
    pub fn name_ident(&self) -> &syn::Ident {
        &self.name_ident
    }
}

impl syn::parse::Parse for MbeHeader {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attributes = syn::Attribute::parse_outer(input)?;
        let vis: syn::Visibility = input.parse()?;
        let macro_keyword = {
            if input.peek(syn::Token![macro]) {
                let macro_keyword: syn::Token![macro] = input.parse().unwrap();
                MacroKeyword::Macro(macro_keyword)
            } else {
                let macro_ident: syn::Ident = input.parse()?;
                let macro_ident_string = macro_ident.to_string();
                if macro_ident_string == "macro_rules" {
                    let excl_token: syn::Token![!] = input.parse()?;
                    MacroKeyword::MacroRules(macro_ident, excl_token)
                } else {
                    return Err(syn::Error::new(macro_ident.span(), "unexpected"));
                }
            }
        };
        let name_ident: syn::Ident = input.parse()?;

        Ok(MbeHeader {
            attributes,
            vis,
            macro_keyword,
            name_ident,
        })
    }
}

impl quote::ToTokens for MbeHeader {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in &self.attributes {
            attr.to_tokens(tokens);
        }
        self.vis.to_tokens(tokens);
        match &self.macro_keyword {
            MacroKeyword::MacroRules(ident, excl_token) => {
                ident.to_tokens(tokens);
                excl_token.to_tokens(tokens);
            }
            MacroKeyword::Macro(ident) => ident.to_tokens(tokens),
        }
        self.name_ident.to_tokens(tokens);
    }
}
