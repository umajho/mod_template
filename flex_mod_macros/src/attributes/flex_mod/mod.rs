pub(crate) mod attribute_options;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use proc_macro2::{Ident, TokenStream, TokenTree};

pub use attribute_options::{
    AttributeOptions, AttributeSubstitutionDeclaration, ConstructionDeclaration,
};

use crate::{
    helper_attributes::{
        __CONSTRUCT::AttributeOptions as ConstructHelperAttributeOptions,
        __SUBSTITUTE::AttributeOptions as SubstituteHelperAttributeOptions,
    },
    utils::substitute_attributes::{substitute_attributes, Substituter},
};

pub fn flex_mod(attr: TokenStream, item: TokenStream) -> TokenStream {
    check_top_mod_on_error_abort(item.clone());
    let opts: AttributeOptions = match syn::parse2(attr.clone()) {
        Ok(attr) => attr,
        Err(err) => return err.to_compile_error(),
    };
    let macro_name_ident = opts.macro_name_ident();

    let output_item = match check_helper_attributes(item, &opts) {
        Ok(output) => output,
        Err(errors) => {
            let mut output = TokenStream::new();
            for error in errors {
                output.extend(error.to_compile_error());
            }
            return output;
        }
    };

    quote::quote! {
        macro_rules! #macro_name_ident {
            ($($input:tt)*) => {
                #[::flex_mod::__monomorphize_mod(
                    // TODO: just pass tokens after `;` in `attr`, since the
                    // macro name is unnecessary for the attribute
                    // `__monomorphize_mod`.
                    //
                    // Or we can also implement `ToTokens` for `*Declaration`,
                    // and manually put those declarations required by
                    // `__monomorphize_mod` here, with commas to separate them.
                    // (`(#constructions, #attribute_substitutions),`)
                    (#attr),
                    { $($input)* }
                )]
                #output_item
            };
        }
    }
}

fn check_top_mod_on_error_abort(input_item: TokenStream) {
    let mut has_found_mod = false;
    let mut mod_name_ident: Option<Ident> = None;
    for tt in input_item {
        if let TokenTree::Punct(punct) = &tt {
            if punct.as_char() == '#' {
                // NOTE: that's because one can also put attributes in the mod
                // header and the order of attributes from those two places
                // would be ambiguous.
                proc_macro_error::abort_call_site!(
                    "attributes directly below the `flex_mod` attribute are not allowed"
                );
            }
        }
        if !has_found_mod {
            if let TokenTree::Ident(ident) = &tt {
                if ident == "mod" {
                    has_found_mod = true
                }
            }
        } else if mod_name_ident.is_none() {
            let TokenTree::Ident(ident) = &tt else {
                proc_macro_error::abort!(tt.span(), "unexpected");
            };
            mod_name_ident = Some(ident.clone())
        }
    }

    if !has_found_mod {
        proc_macro_error::abort_call_site!("this attribute can only be applied to a `mod`");
    }
    let Some(mod_name_ident) = mod_name_ident else {
        proc_macro_error::abort_call_site!("what, a module without a name?");
    };

    if mod_name_ident != "__" {
        proc_macro_error::abort!(
            mod_name_ident.span(),
            format!(
                "{} {}",
                "this should always be `__`, to emphasis that",
                "the `flex_mod` attribute nullifies the name of the module it applied to"
            )
        )
    }
}

fn check_helper_attributes(
    input_item: TokenStream,
    opts: &AttributeOptions,
) -> Result<TokenStream, syn::Error> {
    let constructions = {
        let constructions = opts
            .constructions()
            .iter()
            .map(|item| item.target_name_ident().to_string());
        let constructions: HashSet<_> = HashSet::from_iter(constructions);
        Rc::new(constructions)
    };

    let attribute_substitutions = {
        let attribute_substitutions = opts
            .attribute_substitutions()
            .iter()
            .map(|item| item.target_name_ident().to_string());
        let attribute_substitutions: HashSet<_> = HashSet::from_iter(attribute_substitutions);
        Rc::new(attribute_substitutions)
    };

    let mut attr_map: HashMap<String, Box<Substituter>> = HashMap::new();
    attr_map.insert(
        "__CONSTRUCT".to_string(),
        Box::new(move |meta| {
            let meta = meta.require_list()?;
            let opts: ConstructHelperAttributeOptions = syn::parse2(meta.tokens.clone())?;

            for construction in opts.constructions() {
                let target_name_ident = construction.target_name_ident();
                let target_name = target_name_ident.to_string();
                if !constructions.contains(&target_name) {
                    return Err(syn::Error::new(
                        target_name_ident.span(),
                        format!(
                            "unknown target name `{}`. {} {}",
                            target_name,
                            "It should be declared in the `constructions` block",
                            "among the options of the `flex_mod` attribute"
                        ),
                    ));
                }
            }

            Ok(quote::quote!(#[#meta]))
        }),
    );
    attr_map.insert(
        "__SUBSTITUTE".to_string(),
        Box::new(move |meta| {
            let meta = meta.require_list()?;
            let opts: SubstituteHelperAttributeOptions = syn::parse2(meta.tokens.clone())?;

            let target_name_ident = opts.target_name_ident();
            let target_name = target_name_ident.to_string();
            if !attribute_substitutions.contains(&target_name) {
                return Err(syn::Error::new(
                    target_name_ident.span(),
                    format!(
                        "unknown target name `{}`. {} {}",
                        target_name,
                        "It should be declared in the `attribute_substitutions` block",
                        "among the options of the `flex_mod` attribute"
                    ),
                ));
            }

            Ok(quote::quote!(#[#meta]))
        }),
    );

    substitute_attributes(input_item, &mut attr_map)
}

#[cfg(test)]
mod tests {
    use super::flex_mod;

    #[test]
    fn basic() {
        let input_attr =
            quote::quote!(the_macro_name; constructions(FOO), attribute_substitutions(BAR));
        let input_item = quote::quote! {
            mod __ {
                #[__CONSTRUCT(foo as FOO)]
                fn an_fn() {}

                fn a_second_fn() {}

                mod a_sub_mod {
                    #[__SUBSTITUTE(BAR)]
                    fn a_third_fn() {}
                }

                #[__CONSTRUCT(foo as FOO)]
                #[__SUBSTITUTE(BAR)]
                fn a_fourth_fn() {}
            }
        };

        let expected = quote::quote! {
            macro_rules! the_macro_name {
                ($($input:tt)*) => {
                    #[::flex_mod::__monomorphize_mod(
                        (the_macro_name; constructions(FOO), attribute_substitutions(BAR)),
                        { $($input)* }
                    )]
                    mod __ {
                        #[__CONSTRUCT(foo as FOO)]
                        fn an_fn() {}

                        fn a_second_fn() {}

                        mod a_sub_mod {
                            #[__SUBSTITUTE(BAR)]
                            fn a_third_fn() {}
                        }

                        #[__CONSTRUCT(foo as FOO)]
                        #[__SUBSTITUTE(BAR)]
                        fn a_fourth_fn() {}
                    }
                };
            }
        };

        let actual = flex_mod(input_attr, input_item);

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
