pub(crate) mod attribute_substitution_declaration;
pub(crate) mod construction_declaration;
mod mod_header;

pub use self::attribute_substitution_declaration::AttributeSubstitutionDefinition;
pub use self::construction_declaration::ConstructionDefinition;

use self::mod_header::ModHeader;

pub struct AttributeOptions {
    mod_header: ModHeader,
    constructions: Vec<ConstructionDefinition>,
    attribute_substitutions: Vec<AttributeSubstitutionDefinition>,
}

impl AttributeOptions {
    pub fn mod_header(&self) -> &ModHeader {
        &self.mod_header
    }
    pub fn constructions(&self) -> &Vec<ConstructionDefinition> {
        &self.constructions
    }
    pub fn attribute_substitutions(&self) -> &Vec<AttributeSubstitutionDefinition> {
        &self.attribute_substitutions
    }
}

impl syn::parse::Parse for AttributeOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mod_header: ModHeader = input.parse()?;
        if input.is_empty() {
            return Ok(Self {
                mod_header,
                constructions: vec![],
                attribute_substitutions: vec![],
            });
        }
        let _: syn::Token![;] = input.parse()?;

        let mut constructions: Option<Vec<ConstructionDefinition>> = None;
        let mut attribute_substitutions: Option<Vec<AttributeSubstitutionDefinition>> = None;

        loop {
            if input.is_empty() {
                return Ok(Self {
                    mod_header,
                    constructions: constructions.unwrap_or_default(),
                    attribute_substitutions: attribute_substitutions.unwrap_or_default(),
                });
            }

            let ident: syn::Ident = input.parse()?;
            match &ident.to_string()[..] {
                "constructions" => {
                    if constructions.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "duplicate constructions block",
                        ));
                    }

                    constructions = Some(construction_declaration::parse(input)?);
                }
                "attribute_substitutions" => {
                    if attribute_substitutions.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "duplicate attribute-substitutions block",
                        ));
                    }

                    attribute_substitutions =
                        Some(attribute_substitution_declaration::parse(input)?);
                }
                _ => {
                    return Err(syn::Error::new(ident.span(), "unexpected"));
                }
            }

            if !input.is_empty() {
                let _: syn::Token![,] = input.parse()?;
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use proc_macro2::TokenStream;
    use quote::ToTokens;

    use crate::attributes::extend_parameter_list;

    use super::{
        attribute_substitution_declaration::tests::AttributeSubstitutionDefinitionForTest,
        construction_declaration::tests::ConstructionDefinitionForTest, AttributeOptions,
    };

    #[derive(Debug, PartialEq, Eq, typed_builder::TypedBuilder)]
    pub struct AttributeOptionsForTest {
        mod_header: String,
        constructions: Vec<ConstructionDefinitionForTest>,
        attribute_substitutions: Vec<AttributeSubstitutionDefinitionForTest>,
    }

    impl From<AttributeOptions> for AttributeOptionsForTest {
        fn from(value: AttributeOptions) -> Self {
            let AttributeOptions {
                mod_header,
                constructions,
                attribute_substitutions: attr_subst,
            } = value;

            Self {
                mod_header: mod_header.into_token_stream().to_string(),
                constructions: constructions.into_iter().map(|c| c.into()).collect(),
                attribute_substitutions: attr_subst.into_iter().map(|c| c.into()).collect(),
            }
        }
    }

    fn fixture_mod_header() -> TokenStream {
        quote::quote!(
            #[an_attr]
            #[another_attr]
            pub mod a_mod
        )
    }

    #[test]
    fn basic() {
        let mod_header = fixture_mod_header();
        let expr_foo = quote::quote!("foo");
        let expr_bar = quote::quote!("bar");
        let attr_baz = quote::quote!(#[baz]);
        let attr_qux = quote::quote!(#[qux]);
        let param_list_qux = quote::quote!(qux: Qux);

        let input = quote::quote!(
            #mod_header;
            constructions{ FOO => #expr_foo, BAR => #expr_bar },
            attribute_substitutions{ BAZ => #attr_baz, QUX => #attr_qux (.., #param_list_qux) },
        );

        let expected = AttributeOptionsForTest::builder()
            .mod_header(mod_header.to_string())
            .constructions(vec![
                ConstructionDefinitionForTest::builder()
                    .target_name("FOO".to_string())
                    .construction(expr_foo.to_string())
                    .build(),
                ConstructionDefinitionForTest::builder()
                    .target_name("BAR".to_string())
                    .construction(expr_bar.to_string())
                    .build(),
            ])
            .attribute_substitutions(vec![
                AttributeSubstitutionDefinitionForTest::builder()
                    .target_name("BAZ".to_string())
                    .new_attributes(vec![attr_baz.to_string()])
                    .parameter_list_extension(None)
                    .build(),
                AttributeSubstitutionDefinitionForTest::builder()
                    .target_name("QUX".to_string())
                    .new_attributes(vec![attr_qux.to_string()])
                    .parameter_list_extension(Some(
                        extend_parameter_list::tests::AttributeOptionsForTest::builder()
                            .direction(extend_parameter_list::Direction::Append)
                            .parameter_list(param_list_qux.to_string())
                            .build(),
                    ))
                    .build(),
            ])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }

    #[test]
    fn only_mod_header() {
        let mod_header = fixture_mod_header();

        let input = quote::quote!(#mod_header);

        let expected = AttributeOptionsForTest::builder()
            .mod_header(mod_header.to_string())
            .constructions(vec![])
            .attribute_substitutions(vec![])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }

    #[test]
    fn no_constructions_and_without_trailing_comma() {
        let mod_header = fixture_mod_header();
        let attr_baz = quote::quote!(#[baz]);

        let input = quote::quote!(#mod_header; attribute_substitutions{BAZ => #attr_baz});

        let expected = AttributeOptionsForTest::builder()
            .mod_header(mod_header.to_string())
            .constructions(vec![])
            .attribute_substitutions(vec![AttributeSubstitutionDefinitionForTest::builder()
                .target_name("BAZ".to_string())
                .new_attributes(vec![attr_baz.to_string()])
                .parameter_list_extension(None)
                .build()])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }

    #[test]
    fn no_attribute_substitutions_and_with_no_parameters_in_constructions() {
        let mod_header = fixture_mod_header();

        let input = quote::quote!(#mod_header; constructions{});

        let expected = AttributeOptionsForTest::builder()
            .mod_header(mod_header.to_string())
            .constructions(vec![])
            .attribute_substitutions(vec![])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }
}
