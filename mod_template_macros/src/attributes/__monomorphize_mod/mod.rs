pub(crate) mod attribute_options;
mod attribute_options_pair;

use std::{collections::HashMap, rc::Rc};

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;

use crate::{
    attributes::__monomorphize_mod::attribute_options_pair::AttributeOptionsPair,
    helper_attributes::{
        __CONSTRUCT::AttributeOptions as ConstructHelperAttributeOptions,
        __SUBSTITUTE::AttributeOptions as SubstituteHelperAttributeOptions,
    },
    utils::substitute_attributes::{substitute_attributes, Substituter},
};

pub use self::attribute_options::AttributeOptions;

pub fn __monomorphize_mod(attr: TokenStream, item: TokenStream) -> TokenStream {
    let opts_pair: AttributeOptionsPair = match syn::parse2(attr) {
        Ok(pair) => pair,
        Err(err) => return err.to_compile_error(),
    };
    if let Err(err) = opts_pair.validate() {
        let mut output = TokenStream::new();
        output.extend(err.to_compile_error());

        return output;
    }

    let mod_items = {
        let mod_group = (item.into_iter().last())
            .expect("the attribute `mod_template::define` should guarantee that the last element of the input item stream exists");
        let TokenTree::Group(mod_group) = mod_group else {
            panic!("the attribute `mod_template::define` should guarantee that the last element of the input item is a group");
        };
        mod_group.stream()
    };
    let opts = opts_pair.__monomorphize_mod();
    let type_map = {
        let mut type_map: HashMap<String, syn::Type> = HashMap::new();
        for construction in opts_pair.define().constructions() {
            type_map.insert(
                construction.target_name_ident().to_string(),
                construction.ty().clone(),
            );
        }
        type_map
    };
    let output_items = match monomorphize_items(mod_items, opts, type_map) {
        Ok(output_items) => output_items,
        Err(errs) => {
            // TODO: DRY
            let mut output = TokenStream::new();
            for err in errs {
                output.extend(err.to_compile_error());
            }
            return output;
        }
    };
    let mod_header = opts.mod_header().to_token_stream();

    let output = quote::quote! { #mod_header { #output_items } };

    output.to_token_stream()
}

const EXPECT_AVAILABLE: &str = "the availability of the definition should already be checked by calling `opts_pair.validate()` in the outer function";
const EXPECT_CONSTRUCTION_TYPE_AVAILABLE: &str =
    "any available construction should have a corresponding type";

fn monomorphize_items<'a>(
    input_item: TokenStream,
    opts: &'a AttributeOptions,
    type_map: HashMap<String, syn::Type>,
) -> Result<TokenStream, syn::Error> {
    let opts = Rc::new(opts);
    let opts_for_construct = opts.clone();
    let opts_for_substitute = opts.clone();

    let mut attr_map: HashMap<String, Box<Substituter<'a>>> = HashMap::new();
    attr_map.insert(
        "__CONSTRUCT".to_string(),
        Box::new(move |meta| {
            let opts = &opts_for_construct;

            let meta = meta.require_list()?;
            let helper_opts: ConstructHelperAttributeOptions = syn::parse2(meta.tokens.clone())?;

            let mut output = TokenStream::new();
            for construction in helper_opts.constructions() {
                let target_name = construction.target_name_ident().to_string();
                let def = opts
                    .constructions()
                    .iter()
                    .find(|x| *x.target_name_ident() == target_name)
                    .expect(EXPECT_AVAILABLE);
                let pattern_to_construct = construction.pattern_to_construct();
                let ty = type_map
                    .get(&target_name)
                    .expect(EXPECT_CONSTRUCTION_TYPE_AVAILABLE);
                let construction = def.construction();
                quote::quote!(
                    #[::mod_template::construct(#pattern_to_construct: #ty = #construction)])
                .to_tokens(&mut output);
            }

            Ok(output)
        }),
    );
    attr_map.insert(
        "__SUBSTITUTE".to_string(),
        Box::new(move |meta| {
            let opts = &opts_for_substitute;

            let meta = meta.require_list()?;
            let helper_opts: SubstituteHelperAttributeOptions = syn::parse2(meta.tokens.clone())?;

            let mut output = TokenStream::new();
            let def = opts
                .attribute_substitutions()
                .iter()
                .find(|x| *x.target_name_ident() == *helper_opts.target_name_ident())
                .expect(EXPECT_AVAILABLE);
            if let Some(ext) = def.parameter_list_extension() {
                quote::quote!(#[::mod_template::extend_parameter_list(#ext)])
                    .to_tokens(&mut output);
            }
            for new_attribute in def.new_attributes() {
                new_attribute.to_tokens(&mut output)
            }

            Ok(output)
        }),
    );

    substitute_attributes(input_item, &mut attr_map)
}

#[cfg(test)]
mod tests {
    use super::__monomorphize_mod;

    #[test]
    fn basic() {
        let input_attr = quote::quote!(
            (macro_name; constructions(CONS -> impl ToCons), attribute_substitutions(ATTR_SUBST)),
            {
                mod a_mod;
                constructions {
                    CONS => new_something(),
                },
                attribute_substitutions {
                    ATTR_SUBST => #[an_attr] (.., a_param: AType),
                },
            }
        );
        let input_item = quote::quote! {
            mod __ {
                #[__CONSTRUCT(to_construct as CONS)]
                #[__SUBSTITUTE(ATTR_SUBST)]
                fn an_fn() {}
            }
        };

        let expected = quote::quote! {
            mod a_mod {
                #[::mod_template::construct(to_construct: impl ToCons = new_something())]
                #[::mod_template::extend_parameter_list(.., a_param: AType)]
                #[an_attr]
                fn an_fn() {}
            }
        };

        let actual = __monomorphize_mod(input_attr, input_item);

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
