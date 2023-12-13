use std::collections::HashSet;

pub struct ConstructionDeclaration {
    target_name_ident: syn::Ident,
}

impl ConstructionDeclaration {
    pub fn target_name_ident(&self) -> &syn::Ident {
        &self.target_name_ident
    }
}

pub fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<ConstructionDeclaration>> {
    fn parse_param(input: syn::parse::ParseStream) -> syn::Result<ConstructionDeclaration> {
        let target_name_ident: syn::Ident = input.parse()?;

        Ok(ConstructionDeclaration { target_name_ident })
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
        for ConstructionDeclaration { target_name_ident } in &vec {
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
    use super::ConstructionDeclaration;

    #[derive(Debug, PartialEq, Eq, typed_builder::TypedBuilder)]
    pub struct ConstructionDeclarationForTest {
        target_name: String,
    }
    impl From<ConstructionDeclaration> for ConstructionDeclarationForTest {
        fn from(value: ConstructionDeclaration) -> Self {
            let ConstructionDeclaration { target_name_ident } = value;

            Self {
                target_name: target_name_ident.to_string(),
            }
        }
    }
}
