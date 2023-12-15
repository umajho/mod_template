pub(crate) mod attribute_substitution_declaration;
pub(crate) mod construction_declaration;
pub(crate) mod mbe_header;

use std::collections::HashMap;

pub use self::attribute_substitution_declaration::AttributeSubstitutionDeclaration;
pub use self::construction_declaration::ConstructionDeclaration;
pub use self::mbe_header::MbeHeader;

pub struct AttributeOptions {
    mbe_header: MbeHeader,
    constructions: Vec<ConstructionDeclaration>,
    attribute_substitutions: Vec<AttributeSubstitutionDeclaration>,
}

impl AttributeOptions {
    pub fn mbe_header(&self) -> &MbeHeader {
        &self.mbe_header
    }
    pub fn constructions(&self) -> &Vec<ConstructionDeclaration> {
        &self.constructions
    }
    pub fn attribute_substitutions(&self) -> &Vec<AttributeSubstitutionDeclaration> {
        &self.attribute_substitutions
    }

    pub fn build_type_map(&self) -> HashMap<String, syn::Type> {
        let mut type_map: HashMap<String, syn::Type> = HashMap::new();
        for construction in self.constructions() {
            type_map.insert(
                construction.target_name_ident().to_string(),
                construction.ty().clone(),
            );
        }
        type_map
    }
}

impl syn::parse::Parse for AttributeOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mbe_header: MbeHeader = input.parse()?;
        if input.is_empty() {
            return Ok(Self {
                mbe_header,
                constructions: vec![],
                attribute_substitutions: vec![],
            });
        }
        let _: syn::Token![;] = input.parse()?;

        let mut constructions: Option<Vec<ConstructionDeclaration>> = None;
        let mut attribute_substitutions: Option<Vec<AttributeSubstitutionDeclaration>> = None;

        loop {
            if input.is_empty() {
                return Ok(Self {
                    mbe_header,
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

    use super::{
        attribute_substitution_declaration::tests::AttributeSubstitutionDeclarationForTest,
        construction_declaration::tests::ConstructionDeclarationForTest, AttributeOptions,
    };

    #[derive(Debug, PartialEq, Eq, typed_builder::TypedBuilder)]
    pub struct AttributeOptionsForTest {
        mbe_header: String,
        constructions: Vec<ConstructionDeclarationForTest>,
        attribute_substitutions: Vec<AttributeSubstitutionDeclarationForTest>,
    }
    impl From<AttributeOptions> for AttributeOptionsForTest {
        fn from(value: AttributeOptions) -> Self {
            let AttributeOptions {
                mbe_header,
                constructions,
                attribute_substitutions: attr_subst,
            } = value;

            let mbe_header = mbe_header.into_token_stream().to_string();
            let constructions = constructions.into_iter().map(|x| x.into()).collect();
            let attr_subst = attr_subst.into_iter().map(|x| x.into()).collect();
            Self {
                mbe_header,
                constructions,
                attribute_substitutions: attr_subst,
            }
        }
    }

    #[test]
    fn basic() {
        let input = quote::quote!(
            macro_rules! the_macro_name;
            constructions(FOO -> Foo, BAR -> Bar),
            attribute_substitutions(FOO, BAZ),
        );

        let expected = AttributeOptionsForTest::builder()
            .mbe_header(quote::quote!(macro_rules! the_macro_name).to_string())
            .constructions(vec![
                ConstructionDeclarationForTest::builder()
                    .target_name("FOO".to_string())
                    .ty("Foo".to_string())
                    .build(),
                ConstructionDeclarationForTest::builder()
                    .target_name("BAR".to_string())
                    .ty("Bar".to_string())
                    .build(),
            ])
            .attribute_substitutions(vec![
                AttributeSubstitutionDeclarationForTest::builder()
                    .target_name("FOO".to_string())
                    .build(),
                AttributeSubstitutionDeclarationForTest::builder()
                    .target_name("BAZ".to_string())
                    .build(),
            ])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }

    fn do_test_only_mbe_header(header: TokenStream) {
        let input = quote::quote!(#header);

        let expected = AttributeOptionsForTest::builder()
            .mbe_header(quote::quote!(#header).to_string())
            .constructions(vec![])
            .attribute_substitutions(vec![])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }

    #[test]
    fn only_mbe_header() {
        do_test_only_mbe_header(quote::quote!(macro_rules! the_macro_name));
    }
    #[test]
    fn macro_rules_with_an_attribute() {
        do_test_only_mbe_header(quote::quote!(#[macro_export] macro_rules! the_macro_name));
    }
    #[test]
    fn macro_2_with_pub() {
        do_test_only_mbe_header(quote::quote!(pub macro the_macro_name));
    }

    #[test]
    fn no_constructions_and_without_trailing_comma() {
        let input = quote::quote!(macro_rules! the_macro_name; attribute_substitutions(FOO));

        let expected = AttributeOptionsForTest::builder()
            .mbe_header(quote::quote!(macro_rules! the_macro_name).to_string())
            .constructions(vec![])
            .attribute_substitutions(vec![AttributeSubstitutionDeclarationForTest::builder()
                .target_name("FOO".to_string())
                .build()])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }

    #[test]
    fn no_attribute_substitutions_and_with_no_parameters_in_constructions() {
        let input = quote::quote!(macro_rules! the_macro_name; constructions());

        let expected = AttributeOptionsForTest::builder()
            .mbe_header(quote::quote!(macro_rules! the_macro_name).to_string())
            .constructions(vec![])
            .attribute_substitutions(vec![])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }
}
