use proc_macro2::TokenTree;

pub struct AttributeOptions {
    target_name_ident: syn::Ident,
}
impl AttributeOptions {
    pub fn target_name_ident(&self) -> &syn::Ident {
        &self.target_name_ident
    }
}

impl syn::parse::Parse for AttributeOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let target_name_ident: syn::Ident = input.parse()?;

        if !input.cursor().eof() {
            let tt: TokenTree = input.parse().unwrap();
            return Err(syn::Error::new(tt.span(), "unexpected"));
        }

        Ok(AttributeOptions { target_name_ident })
    }
}
