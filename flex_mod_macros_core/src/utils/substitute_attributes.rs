use std::collections::HashMap;

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::ToTokens;
use syn::Meta;

use super::token_stream_or_syn_errors::TokenStreamOrSynErrors;

pub type Substituter<'a> = dyn Fn(Meta) -> syn::Result<TokenStream> + 'a;

pub fn substitute_attributes(
    input: TokenStream,
    attr_map: &mut HashMap<String, Box<Substituter>>,
) -> Result<TokenStream, Vec<syn::Error>> {
    let mut input = input.into_iter();
    let mut output = TokenStreamOrSynErrors::new();

    'iteration: loop {
        let Some(tt) = input.next() else {
            break output.into_result();
        };

        'process: {
            if let TokenTree::Punct(ref punct) = tt {
                if punct.as_char() != '#' {
                    break 'process;
                }
                let Some(tt_expect_group) = input.next() else {
                    break 'process;
                };
                let TokenTree::Group(ref group) = tt_expect_group else {
                    output.extend([tt, tt_expect_group]);
                    continue 'iteration;
                };
                if group.delimiter() != Delimiter::Bracket {
                    output.extend([tt, tt_expect_group]);
                    continue 'iteration;
                }

                let Some(substituted) = try_substitute_attribute(group.stream(), attr_map) else {
                    output.extend([tt, tt_expect_group]);
                    continue 'iteration;
                };
                match substituted {
                    Ok(substituted) => output.extend(substituted),
                    Err(err) => output.push_error(err),
                }
                continue 'iteration;
            } else if let TokenTree::Group(ref group) = tt {
                let inner_output = substitute_attributes(group.stream(), attr_map);
                match inner_output {
                    Ok(inner_output) => output.append(Group::new(group.delimiter(), inner_output)),
                    Err(inner_errors) => output.extend_errors(inner_errors),
                }

                continue 'iteration;
            }
        }

        output.append(tt);
    }
}

fn try_substitute_attribute(
    meta: TokenStream,
    attr_map: &mut HashMap<String, Box<Substituter>>,
) -> Option<syn::Result<TokenStream>> {
    let meta: Meta = syn::parse2(meta).ok()?;

    let path = meta.path().into_token_stream().to_string();
    let Some(ref substituter) = attr_map.get(&path) else {
        return None;
    };

    Some(substituter(meta))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::utils::substitute_attributes::Substituter;

    use super::substitute_attributes;

    #[test]
    fn basic() {
        let input = quote::quote! {
            #[foo]
            mod a_mod {
                #[foo]
                #[not_in_map]
                #[bar]
                struct a_struct {
                    #[baz(#[zab])]
                    a_field: i32,
                }
                impl a_struct {
                    #[bar]
                    #[foo]
                    fn an_fn() {}
                }
            }
        };

        let mut attr_map: HashMap<String, Box<Substituter>> = HashMap::new();
        attr_map.insert("foo".to_string(), Box::new(|_| Ok(quote::quote!(#[oof]))));
        attr_map.insert(
            "bar".to_string(),
            Box::new(|_| Ok(quote::quote!(#[bar_1] #[bar_2]))),
        );
        attr_map.insert(
            "baz".to_string(),
            Box::new(|meta| Ok(meta.require_list().unwrap().tokens.clone())),
        );

        let expected = quote::quote! {
            #[oof]
            mod a_mod {
                #[oof]
                #[not_in_map]
                #[bar_1]
                #[bar_2]
                struct a_struct {
                    #[zab]
                    a_field: i32,
                }
                impl a_struct {
                    #[bar_1]
                    #[bar_2]
                    #[oof]
                    fn an_fn() {}
                }
            }
        };

        let actual = substitute_attributes(input, &mut attr_map);

        assert_eq!(
            expected.to_string(),
            actual.expect("there should not be errors").to_string()
        )
    }

    mod errors {
        use super::*;

        fn before_each() -> (HashMap<String, Box<Substituter<'static>>>,) {
            let mut attr_map: HashMap<String, Box<Substituter>> = HashMap::new();
            attr_map.insert(
                "errored".to_string(),
                Box::new(|meta| {
                    let error_span = meta.path().segments.last().unwrap().ident.span();
                    Err(syn::Error::new(error_span, "errored"))
                }),
            );

            (attr_map,)
        }

        #[test]
        fn single() {
            let (mut attr_map,) = before_each();

            let input = quote::quote! {
                #[errored]
                mod foo {}
            };

            let actual = substitute_attributes(input, &mut attr_map);

            let actual_errors = actual.expect_err("should have an error");

            assert_eq!(actual_errors.len(), 1);
            assert_eq!(actual_errors[0].to_string(), "errored");
        }

        #[test]
        fn multiple() {
            let (mut attr_map,) = before_each();

            let input = quote::quote! {
                #[errored]
                mod foo {
                    #[errored]
                    fn bar() {}
                }
            };

            let actual = substitute_attributes(input, &mut attr_map);

            let actual_errors = actual.expect_err("should have an error");

            assert_eq!(actual_errors.len(), 2);
            for error in actual_errors {
                assert_eq!(error.to_string(), "errored");
            }
        }
    }
}
