use std::collections::HashSet;

use syn::parse::Parse;

use crate::attributes::extend_parameter_list;

pub struct AttributeSubstitutionDefinition {
    target_name_ident: syn::Ident,
    new_attributes: Vec<syn::Attribute>,
    parameter_list_extension: Option<extend_parameter_list::AttributeOptions>,
}

impl AttributeSubstitutionDefinition {
    pub fn target_name_ident(&self) -> &syn::Ident {
        &self.target_name_ident
    }

    pub fn new_attributes(&self) -> &Vec<syn::Attribute> {
        &self.new_attributes
    }

    pub fn parameter_list_extension(&self) -> &Option<extend_parameter_list::AttributeOptions> {
        &self.parameter_list_extension
    }
}

pub fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<AttributeSubstitutionDefinition>> {
    fn parse_param(input: syn::parse::ParseStream) -> syn::Result<AttributeSubstitutionDefinition> {
        let target_name_ident: syn::Ident = input.parse()?;
        let _: syn::Token![=>] = input.parse()?;
        let new_attributes = input.call(syn::Attribute::parse_outer)?;
        let parameter_list_extension = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Some(content.call(extend_parameter_list::AttributeOptions::parse)?)
        } else {
            None
        };

        if new_attributes.is_empty() && parameter_list_extension.is_none() {
            return Err(syn::Error::new(
                target_name_ident.span(),
                format!(
                    "{} `{} => (..)`, {}",
                    "consider rewriting this entry as",
                    target_name_ident,
                    "to make the right side of the arrow not be empty"
                ),
            ));
        }

        let parameter_list_extension =
            parameter_list_extension.and_then(|x| if x.is_noop() { None } else { Some(x) });

        Ok(AttributeSubstitutionDefinition {
            target_name_ident,
            new_attributes,
            parameter_list_extension,
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
        for AttributeSubstitutionDefinition {
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
    use std::vec;

    use quote::ToTokens;
    use syn::parse::Parser;

    use crate::attributes::extend_parameter_list;

    use super::{parse, AttributeSubstitutionDefinition};

    #[derive(Debug, PartialEq, Eq, typed_builder::TypedBuilder)]
    pub struct AttributeSubstitutionDefinitionForTest {
        target_name: String,
        new_attributes: Vec<String>,
        parameter_list_extension: Option<extend_parameter_list::tests::AttributeOptionsForTest>,
    }
    impl From<AttributeSubstitutionDefinition> for AttributeSubstitutionDefinitionForTest {
        fn from(value: AttributeSubstitutionDefinition) -> Self {
            let AttributeSubstitutionDefinition {
                target_name_ident,
                new_attributes,
                parameter_list_extension,
            } = value;

            Self {
                target_name: target_name_ident.to_string(),
                new_attributes: new_attributes
                    .into_iter()
                    .map(|attr| attr.into_token_stream().to_string())
                    .collect(),
                parameter_list_extension: parameter_list_extension.map(|x| x.into()),
            }
        }
    }

    #[test]
    fn basic() {
        let attr_foo = quote::quote!(#[foo]);
        let attr_bar = quote::quote!(#[bar]);
        let param_list_baz = quote::quote!(baz_1: Baz, mut baz_2: &mut Baz);

        let input = quote::quote!({
            WITH_ATTRS => #attr_foo #attr_bar,
            WITH_EXT => (.., #param_list_baz),
            WITH_ATTRS_EXT => #attr_foo #attr_bar (.., #param_list_baz),
            WITH_EMPTY_EXT => (..),
        });

        let expected = vec![
            AttributeSubstitutionDefinitionForTest::builder()
                .target_name("WITH_ATTRS".to_string())
                .new_attributes(vec![attr_foo.to_string(), attr_bar.to_string()])
                .parameter_list_extension(None)
                .build(),
            AttributeSubstitutionDefinitionForTest::builder()
                .target_name("WITH_EXT".to_string())
                .new_attributes(vec![])
                .parameter_list_extension(Some(
                    extend_parameter_list::tests::AttributeOptionsForTest::builder()
                        .direction(extend_parameter_list::Direction::Append)
                        .parameter_list(param_list_baz.to_string())
                        .build(),
                ))
                .build(),
            AttributeSubstitutionDefinitionForTest::builder()
                .target_name("WITH_ATTRS_EXT".to_string())
                .new_attributes(vec![attr_foo.to_string(), attr_bar.to_string()])
                .parameter_list_extension(Some(
                    extend_parameter_list::tests::AttributeOptionsForTest::builder()
                        .direction(extend_parameter_list::Direction::Append)
                        .parameter_list(param_list_baz.to_string())
                        .build(),
                ))
                .build(),
            AttributeSubstitutionDefinitionForTest::builder()
                .target_name("WITH_EMPTY_EXT".to_string())
                .new_attributes(vec![])
                .parameter_list_extension(None)
                .build(),
        ];

        let actual = (|input: syn::parse::ParseStream| parse(input))
            .parse2(input)
            .expect("should be able to parse the input");
        let actual: Vec<AttributeSubstitutionDefinitionForTest> =
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
        let actual: Vec<AttributeSubstitutionDefinitionForTest> =
            actual.into_iter().map(|x| x.into()).collect();

        assert_eq!(actual, expected)
    }
}
