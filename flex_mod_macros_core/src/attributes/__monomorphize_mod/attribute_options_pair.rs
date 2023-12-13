use proc_macro2::TokenTree;

use crate::attributes::{__monomorphize_mod, flex_mod};

pub struct AttributeOptionsPair(
    flex_mod::AttributeOptions,
    __monomorphize_mod::AttributeOptions,
);

impl AttributeOptionsPair {
    pub fn flex_mod(&self) -> &flex_mod::AttributeOptions {
        &self.0
    }

    pub fn __monomorphize_mod(&self) -> &__monomorphize_mod::AttributeOptions {
        &self.1
    }
}

impl syn::parse::Parse for AttributeOptionsPair {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let flex_mod_attr_opts: flex_mod::AttributeOptions = {
            let content;
            syn::parenthesized!(content in input);
            content.parse()?
        };

        let _: syn::Token![,] = input.parse()?;

        let monomorphize_mod_attr_opts: __monomorphize_mod::AttributeOptions = {
            let content;
            syn::braced!(content in input);
            content.parse()?
        };

        if !input.cursor().eof() {
            let tt: TokenTree = input.parse().unwrap();
            return Err(syn::Error::new(tt.span(), "unexpected"));
        }

        Ok(Self(flex_mod_attr_opts, monomorphize_mod_attr_opts))
    }
}

impl AttributeOptionsPair {
    pub fn validate(&self) -> Result<(), Vec<syn::Error>> {
        macro_rules! map_to_ident {
            ($expr:expr) => {
                $expr
                    .iter()
                    .map(|item| item.target_name_ident().clone())
                    .collect::<Vec<syn::Ident>>()
            };
        }

        let declared_constructions = map_to_ident!(self.flex_mod().constructions());
        let declared_attr_substs = map_to_ident!(self.flex_mod().attribute_substitutions());
        let defined_constructions = map_to_ident!(self.__monomorphize_mod().constructions());
        let defined_attr_substs =
            map_to_ident!(self.__monomorphize_mod().attribute_substitutions());

        let (undecl_constructions, undef_constructions) =
            utils::diff_by_display(&declared_constructions, &defined_constructions);
        let (undecl_attr_substs, undef_attr_substs) =
            utils::diff_by_display(&declared_attr_substs, &defined_attr_substs);

        let mut errs: Vec<syn::Error> = Vec::new();
        #[derive(PartialEq, Eq)]
        enum What {
            Undecl,
            Undef,
        }
        for (what, which, unknown_target_names_ident) in [
            (What::Undecl, "constructions", undecl_constructions),
            (What::Undecl, "attribute_substitutions", undecl_attr_substs),
            (What::Undef, "constructions", undef_constructions),
            (What::Undef, "attribute_substitutions", undef_attr_substs),
        ] {
            for target_name_ident in unknown_target_names_ident {
                let message = if what == What::Undecl {
                    format!(
                        "unknown target name `{}`. {}{}{} {}",
                        target_name_ident,
                        "It should be declared in the `",
                        which,
                        "` block",
                        "among the options of the `flex_mod` attribute"
                    )
                } else {
                    format!("missing target name `{}` in {}", target_name_ident, which)
                };
                errs.push(syn::Error::new(target_name_ident.span(), message))
            }
        }

        Ok(())
    }
}

mod utils {
    use std::{
        collections::{hash_map::RandomState, HashMap, HashSet},
        fmt::Display,
        hash::Hash,
    };

    pub fn diff_by_display<'a, Item>(
        lefts: &'a [Item],
        rights: &'a [Item],
    ) -> (Vec<&'a Item>, Vec<&'a Item>)
    where
        Item: Display + Hash + Eq,
    {
        let l_map = build_display_to_item_map(lefts);
        let r_map = build_display_to_item_map(rights);

        let l_set: HashSet<&String, RandomState> = HashSet::from_iter(l_map.keys());
        let r_set: HashSet<&String, RandomState> = HashSet::from_iter(r_map.keys());

        let r_items_absent_in_l = r_set.difference(&l_set);
        let l_items_absent_in_r = l_set.difference(&r_set);

        (
            r_items_absent_in_l
                .into_iter()
                .map(|item| *r_map.get(*item).unwrap())
                .collect(),
            l_items_absent_in_r
                .into_iter()
                .map(|item| *l_map.get(*item).unwrap())
                .collect(),
        )
    }

    fn build_display_to_item_map<T: Display>(items: &[T]) -> HashMap<String, &T> {
        items.iter().map(|item| (item.to_string(), item)).collect()
    }

    #[cfg(test)]
    mod tests {
        use std::collections::HashSet;

        use super::diff_by_display;

        #[test]
        fn it_works() {
            let primes = [2, 3, 5, 7, 11, 13, 17, 19];
            let odds = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19];

            let (odds_not_prime, primes_not_odd) = diff_by_display(&primes, &odds);

            assert_eq!(
                odds_not_prime.into_iter().copied().collect::<HashSet<_>>(),
                HashSet::from([1, 9, 15])
            );
            assert_eq!(
                primes_not_odd.into_iter().copied().collect::<HashSet<_>>(),
                HashSet::from([2])
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::attributes::{
        __monomorphize_mod::attribute_options::{
            attribute_substitution_declaration::tests::AttributeSubstitutionDefinitionForTest,
            construction_declaration::tests::ConstructionDefinitionForTest,
            tests::AttributeOptionsForTest as MonomorphizeModAttributeOptionsForTest,
        },
        extend_parameter_list::{
            tests::AttributeOptionsForTest as ExtendParameterListAttributeOptionsForTest, Direction,
        },
        flex_mod::attribute_options::{
            attribute_substitution_declaration::tests::AttributeSubstitutionDeclarationForTest,
            construction_declaration::tests::ConstructionDeclarationForTest,
            tests::AttributeOptionsForTest as FlexModAttributeOptionsForTest,
        },
    };

    use super::AttributeOptionsPair;

    #[derive(Debug, PartialEq, Eq)]
    struct AttributeOptionsPairForTest(
        FlexModAttributeOptionsForTest,
        MonomorphizeModAttributeOptionsForTest,
    );

    impl From<AttributeOptionsPair> for AttributeOptionsPairForTest {
        fn from(value: AttributeOptionsPair) -> Self {
            Self(value.0.into(), value.1.into())
        }
    }

    #[test]
    fn basic() {
        let mod_header = quote::quote!(
            pub mod a_mod
        );
        let expr_new_something = quote::quote!(new_something());
        let attr_substs = quote::quote!(#[an_attr]);
        let param_list = quote::quote!(a_param: AType);

        let input = quote::quote!(
            (macro_name; constructions(CONS), attribute_substitutions(ATTR_SUBST)),
            {
                #mod_header;
                constructions { CONS => #expr_new_something },
                attribute_substitutions { ATTR_SUBST => #attr_substs (.., #param_list) }
            }
        );

        let expected = AttributeOptionsPairForTest(
            FlexModAttributeOptionsForTest::builder()
                .macro_name("macro_name".to_string())
                .constructions(vec![ConstructionDeclarationForTest::builder()
                    .target_name("CONS".to_string())
                    .build()])
                .attribute_substitutions(vec![AttributeSubstitutionDeclarationForTest::builder()
                    .target_name("ATTR_SUBST".to_string())
                    .build()])
                .build(),
            MonomorphizeModAttributeOptionsForTest::builder()
                .mod_header(mod_header.to_string())
                .constructions(vec![ConstructionDefinitionForTest::builder()
                    .target_name("CONS".to_string())
                    .construction(expr_new_something.to_string())
                    .build()])
                .attribute_substitutions(vec![AttributeSubstitutionDefinitionForTest::builder()
                    .target_name("ATTR_SUBST".to_string())
                    .new_attributes(vec![attr_substs.to_string()])
                    .parameter_list_extension(Some(
                        ExtendParameterListAttributeOptionsForTest::builder()
                            .direction(Direction::Append)
                            .parameter_list(param_list.to_string())
                            .build(),
                    ))
                    .build()])
                .build(),
        );

        let actual: AttributeOptionsPair = syn::parse2(input).unwrap();
        let actual: AttributeOptionsPairForTest = actual.into();

        assert_eq!(actual, expected)
    }
}
