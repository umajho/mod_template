use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::__private::ToTokens;

pub fn construct(attr: TokenStream, item: TokenStream) -> TokenStream {
    let opts: AttributeOptions = match syn::parse2(attr) {
        Ok(attr) => attr,
        Err(err) => return err.to_compile_error(),
    };

    let mut output = TokenStream::new();

    let mut has_found_fn = false;
    let mut has_found_body = false;
    'iteration: for tt in Into::<TokenStream>::into(item).into_iter() {
        'process: {
            if !has_found_fn {
                let TokenTree::Ident(ref ident) = tt else {
                    break 'process;
                };
                if *ident == "fn" {
                    has_found_fn = true;
                }
            } else if !has_found_body {
                let TokenTree::Group(ref group) = tt else {
                    break 'process;
                };
                if group.delimiter() != Delimiter::Brace {
                    break 'process;
                }
                has_found_body = true;

                let mut inner_output = TokenStream::new();
                for construction in &opts.constructions {
                    let pattern_to_construct = &construction.pattern_to_construct;
                    let constructor = &construction.constructor;
                    quote::quote! {
                        let #pattern_to_construct = #constructor;
                    }
                    .to_tokens(&mut inner_output);
                }

                inner_output.extend(group.stream());

                output.append(Group::new(Delimiter::Brace, inner_output));

                continue 'iteration;
            }
        }

        output.append(tt);
    }

    if !has_found_fn {
        proc_macro_error::abort_call_site!("this attribute can only be applied to an `fn`");
    } else if !has_found_body {
        proc_macro_error::abort_call_site!("function body not found");
    }

    output
}

struct AttributeOptions {
    constructions: Vec<Construction>,
}

/// `«pattern_to_construct» as «constructor»`.
struct Construction {
    pattern_to_construct: syn::Pat,
    constructor: syn::Expr,
}

impl syn::parse::Parse for AttributeOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let constructions: Vec<_> = input
            .parse_terminated(Construction::parse, syn::Token![,])?
            .into_iter()
            .collect();

        Ok(AttributeOptions { constructions })
    }
}

impl syn::parse::Parse for Construction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern_to_construct = syn::Pat::parse_single(input)?;
        let _: syn::Token![=] = input.parse()?;
        let constructor = input.parse()?;

        Ok(Self {
            pattern_to_construct,
            constructor,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::construct;

    #[test]
    fn basic() {
        let input_attr = quote::quote!(one = 1, mut to_be_three = 2);
        let input_item = quote::quote! {
            fn four() -> i32 {
                to_be_three += 1;
                one + to_be_three
            }
        };

        let expected = quote::quote! {
            fn four() -> i32 {
                let one = 1;
                let mut to_be_three = 2;
                to_be_three += 1;
                one + to_be_three
            }
        };

        let actual = construct(input_attr, input_item);

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn empty() {
        let input_attr = quote::quote!();
        let input_item = quote::quote! {
            fn empty() {}
        };

        let expected = quote::quote! {
            fn empty() {}
        };

        let actual = construct(input_attr, input_item);

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
