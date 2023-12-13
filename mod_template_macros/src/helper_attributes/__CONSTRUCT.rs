pub struct AttributeOptions {
    constructions: Vec<Construction>,
}
impl AttributeOptions {
    pub fn constructions(&self) -> &Vec<Construction> {
        &self.constructions
    }
}

/// `«pattern_to_construct» as «target_name_ident»`
pub struct Construction {
    pattern_to_construct: syn::Pat,
    target_name_ident: syn::Ident,
}
impl Construction {
    pub fn pattern_to_construct(&self) -> &syn::Pat {
        &self.pattern_to_construct
    }
    pub fn target_name_ident(&self) -> &syn::Ident {
        &self.target_name_ident
    }
}

impl syn::parse::Parse for AttributeOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let constructions: Vec<_> = input
            .parse_terminated(Construction::parse, syn::Token![,])?
            .into_iter()
            .collect();

        Ok(AttributeOptions { constructions })
    }
}

impl syn::parse::Parse for Construction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern_to_construct = syn::Pat::parse_single(input)?;
        let _: syn::Token![as] = input.parse()?;
        let target_name_ident = input.parse()?;

        Ok(Self {
            pattern_to_construct,
            target_name_ident,
        })
    }
}
