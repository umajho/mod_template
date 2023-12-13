use std::collections::HashSet;

pub struct AttributeSubstitutionDeclaration {
    target_name_ident: syn::Ident,
}

impl AttributeSubstitutionDeclaration {
    pub fn target_name_ident(&self) -> &syn::Ident {
        &self.target_name_ident
    }
}

pub fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<AttributeSubstitutionDeclaration>> {
    fn parse_param(
        input: syn::parse::ParseStream,
    ) -> syn::Result<AttributeSubstitutionDeclaration> {
        let target_name_ident: syn::Ident = input.parse()?;

        Ok(AttributeSubstitutionDeclaration { target_name_ident })
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
        for AttributeSubstitutionDeclaration { target_name_ident } in &vec {
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
    use super::AttributeSubstitutionDeclaration;

    #[derive(Debug, PartialEq, Eq, typed_builder::TypedBuilder)]
    pub struct AttributeSubstitutionDeclarationForTest {
        target_name: String,
    }
    impl From<AttributeSubstitutionDeclaration> for AttributeSubstitutionDeclarationForTest {
        fn from(value: AttributeSubstitutionDeclaration) -> Self {
            let AttributeSubstitutionDeclaration { target_name_ident } = value;

            Self {
                target_name: target_name_ident.to_string(),
            }
        }
    }
}
