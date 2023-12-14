use std::collections::HashSet;

pub struct ConstructionDeclaration {
    target_name_ident: syn::Ident,
    ty: syn::Type,
}

impl ConstructionDeclaration {
    pub fn target_name_ident(&self) -> &syn::Ident {
        &self.target_name_ident
    }
    pub fn ty(&self) -> &syn::Type {
        &self.ty
    }
}

pub fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<ConstructionDeclaration>> {
    fn parse_param(input: syn::parse::ParseStream) -> syn::Result<ConstructionDeclaration> {
        let target_name_ident: syn::Ident = input.parse()?;
        let _: syn::Token![->] = input.parse()?;
        let ty: syn::Type = input.parse()?;
        if let syn::Type::ImplTrait(syn::TypeImplTrait { impl_token, .. }) = ty {
            return Err(syn::Error::new(
                impl_token.span,
                "`impl` types are unsupported for now",
            ));
        };

        Ok(ConstructionDeclaration {
            target_name_ident,
            ty,
        })
    }

    let vec: Vec<_> = {
        let content;
        syn::parenthesized!(content in input);
        content
            .parse_terminated(parse_param, syn::Token![,])?
            .into_iter()
            .collect()
    };

    {
        let mut previous_names = HashSet::new();
        for ConstructionDeclaration {
            target_name_ident, ..
        } in &vec
        {
            let target_name = target_name_ident.to_string();
            if previous_names.contains(&target_name) {
                return Err(syn::Error::new(
                    target_name_ident.span(),
                    "duplicate target name",
                ));
            }
            previous_names.insert(target_name);
        }
    }

    Ok(vec)
}

#[cfg(test)]
pub(crate) mod tests {
    use quote::ToTokens;

    use super::ConstructionDeclaration;

    #[derive(Debug, PartialEq, Eq, typed_builder::TypedBuilder)]
    pub struct ConstructionDeclarationForTest {
        target_name: String,
        ty: String,
    }
    impl From<ConstructionDeclaration> for ConstructionDeclarationForTest {
        fn from(value: ConstructionDeclaration) -> Self {
            let ConstructionDeclaration {
                target_name_ident,
                ty,
            } = value;

            Self {
                target_name: target_name_ident.to_string(),
                ty: ty.into_token_stream().to_string(),
            }
        }
    }
}
