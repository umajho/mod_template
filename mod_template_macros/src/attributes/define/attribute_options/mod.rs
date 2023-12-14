pub(crate) mod attribute_substitution_declaration;
pub(crate) mod construction_declaration;

use std::collections::HashMap;

use proc_macro2::Ident;

pub use self::attribute_substitution_declaration::AttributeSubstitutionDeclaration;
pub use self::construction_declaration::ConstructionDeclaration;

pub struct AttributeOptions {
    macro_name_ident: Ident,
    constructions: Vec<ConstructionDeclaration>,
    attribute_substitutions: Vec<AttributeSubstitutionDeclaration>,
}

impl AttributeOptions {
    pub fn macro_name_ident(&self) -> &Ident {
        &self.macro_name_ident
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
        let macro_name_ident: syn::Ident = input.parse()?;
        if input.is_empty() {
            return Ok(Self {
                macro_name_ident,
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
                    macro_name_ident,
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
    use super::{
        attribute_substitution_declaration::tests::AttributeSubstitutionDeclarationForTest,
        construction_declaration::tests::ConstructionDeclarationForTest, AttributeOptions,
    };

    #[derive(Debug, PartialEq, Eq, typed_builder::TypedBuilder)]
    pub struct AttributeOptionsForTest {
        macro_name: String,
        constructions: Vec<ConstructionDeclarationForTest>,
        attribute_substitutions: Vec<AttributeSubstitutionDeclarationForTest>,
    }
    impl From<AttributeOptions> for AttributeOptionsForTest {
        fn from(value: AttributeOptions) -> Self {
            let AttributeOptions {
                macro_name_ident,
                constructions,
                attribute_substitutions: attr_subst,
            } = value;

            let macro_name = macro_name_ident.to_string();
            let constructions = constructions.into_iter().map(|x| x.into()).collect();
            let attr_subst = attr_subst.into_iter().map(|x| x.into()).collect();
            Self {
                macro_name,
                constructions,
                attribute_substitutions: attr_subst,
            }
        }
    }

    #[test]
    fn basic() {
        let input = quote::quote!(
            the_macro_name;
            constructions(FOO -> Foo, BAR -> Bar),
            attribute_substitutions(FOO, BAZ),
        );

        let expected = AttributeOptionsForTest::builder()
            .macro_name("the_macro_name".to_string())
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

    #[test]
    fn only_macro_name() {
        let input = quote::quote!(the_macro_name);

        let expected = AttributeOptionsForTest::builder()
            .macro_name("the_macro_name".to_string())
            .constructions(vec![])
            .attribute_substitutions(vec![])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }

    #[test]
    fn no_constructions_and_without_trailing_comma() {
        let input = quote::quote!(the_macro_name; attribute_substitutions(FOO));

        let expected = AttributeOptionsForTest::builder()
            .macro_name("the_macro_name".to_string())
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
        let input = quote::quote!(the_macro_name; constructions());

        let expected = AttributeOptionsForTest::builder()
            .macro_name("the_macro_name".to_string())
            .constructions(vec![])
            .attribute_substitutions(vec![])
            .build();

        let actual: AttributeOptions = syn::parse2(input).unwrap();
        let actual: AttributeOptionsForTest = actual.into();

        assert_eq!(actual, expected);
    }
}
