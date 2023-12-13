use std::collections::HashSet;

pub struct ConstructionDefinition {
    target_name_ident: syn::Ident,
    construction: syn::Expr,
}

impl ConstructionDefinition {
    pub fn target_name_ident(&self) -> &syn::Ident {
        &self.target_name_ident
    }

    pub fn construction(&self) -> &syn::Expr {
        &self.construction
    }
}

pub fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<ConstructionDefinition>> {
    fn parse_param(input: syn::parse::ParseStream) -> syn::Result<ConstructionDefinition> {
        let target_name_ident: syn::Ident = input.parse()?;
        let _: syn::Token![=>] = input.parse()?;
        let construction: syn::Expr = input.parse()?;

        Ok(ConstructionDefinition {
            target_name_ident,
            construction,
        })
    }

    let vec: Vec<_> = {
        let content;
        syn::braced!(content in input);
        content
            .parse_terminated(parse_param, syn::Token![,])?
            .into_iter()
            .collect()
    };

    {
        let mut previous_names = HashSet::new();
        for ConstructionDefinition {
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
    use syn::parse::Parser;

    use super::{parse, ConstructionDefinition};

    #[derive(Debug, PartialEq, Eq, typed_builder::TypedBuilder)]
    pub struct ConstructionDefinitionForTest {
        target_name: String,
        construction: String,
    }
    impl From<ConstructionDefinition> for ConstructionDefinitionForTest {
        fn from(value: ConstructionDefinition) -> Self {
            let ConstructionDefinition {
                target_name_ident,
                construction,
            } = value;

            Self {
                target_name: target_name_ident.to_string(),
                construction: construction.into_token_stream().to_string(),
            }
        }
    }

    #[test]
    fn basic() {
        let expr_foo = quote::quote!(42);
        let expr_bar = quote::quote!({
            let bar = bar();
            bar.do_something()
        });

        let input = quote::quote!({ FOO => #expr_foo, BAR => #expr_bar });

        let expected = vec![
            ConstructionDefinitionForTest::builder()
                .target_name("FOO".to_string())
                .construction(expr_foo.to_string())
                .build(),
            ConstructionDefinitionForTest::builder()
                .target_name("BAR".to_string())
                .construction(expr_bar.to_string())
                .build(),
        ];

        let actual = (|input: syn::parse::ParseStream| parse(input))
            .parse2(input)
            .expect("should be able to parse the input");
        let actual: Vec<ConstructionDefinitionForTest> =
            actual.into_iter().map(|x| x.into()).collect();

        assert_eq!(actual, expected)
    }

    #[test]
    fn empty() {
        let input = quote::quote!({});

        let expected = vec![];

        let actual = (|input: syn::parse::ParseStream| parse(input))
            .parse2(input)
            .expect("should be able to parse the input");
        let actual: Vec<ConstructionDefinitionForTest> =
            actual.into_iter().map(|x| x.into()).collect();

        assert_eq!(actual, expected)
    }
}
