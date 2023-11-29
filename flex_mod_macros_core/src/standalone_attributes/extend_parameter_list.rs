use proc_macro2::{Delimiter, Group, Punct, TokenStream, TokenTree};
use quote::TokenStreamExt;

pub fn extend_parameter_list(attr: TokenStream, item: TokenStream) -> TokenStream {
    let opts: AttributeOptions = match syn::parse2(attr) {
        Ok(attr) => attr,
        Err(err) => return err.to_compile_error(),
    };
    let mut parameter_list = Some(opts.parameter_list);

    let mut output = TokenStream::new();

    let mut has_found_fn = false;
    let mut has_found_param_list = false;
    'iteration: for tt in Into::<TokenStream>::into(item).into_iter() {
        'process: {
            if !has_found_fn {
                let TokenTree::Ident(ref ident) = tt else {
                    break 'process;
                };
                if *ident == "fn" {
                    has_found_fn = true;
                }
            } else if !has_found_param_list {
                let TokenTree::Group(ref group) = tt else {
                    break 'process;
                };
                if group.delimiter() != Delimiter::Parenthesis {
                    break 'process;
                }
                has_found_param_list = true;

                let mut inner_output = TokenStream::new();
                do_extend_parameter_list(
                    group.stream(),
                    &mut inner_output,
                    opts.direction,
                    parameter_list.take().expect("it should only be used once"),
                );

                output.append(Group::new(Delimiter::Parenthesis, inner_output));

                continue 'iteration;
            }
        }

        output.append(tt);
    }

    if !has_found_fn {
        proc_macro_error::abort_call_site!("this attribute can only be applied to an `fn`");
    } else if !has_found_param_list {
        proc_macro_error::abort_call_site!("function parameter list not found");
    }

    output
}

fn do_extend_parameter_list(
    input: TokenStream,
    output: &mut TokenStream,
    direction: Direction,
    parameter_list: Vec<TokenTree>,
) {
    match direction {
        Direction::Append => {
            let mut is_last_comma = true;
            for tt in input.into_iter() {
                is_last_comma = (|| {
                    let TokenTree::Punct(ref punct) = tt else {
                        return false;
                    };
                    punct.as_char() == ','
                })();
                output.append(tt);
            }
            if !is_last_comma {
                output.append(Punct::new(',', proc_macro2::Spacing::Alone));
            }
            output.extend(parameter_list);
        }
    }
}

struct AttributeOptions {
    direction: Direction,
    parameter_list: Vec<TokenTree>,
}

/// Only `append` is supported for now
#[derive(Clone, Copy)]
enum Direction {
    Append,
}

impl syn::parse::Parse for AttributeOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: syn::Token![..] = input.parse()?;
        let parameter_list: TokenStream = if input.cursor().eof() {
            TokenStream::new()
        } else {
            let _: syn::Token![,] = input.parse()?;
            input.parse()?
        };

        Ok(AttributeOptions {
            direction: Direction::Append,
            parameter_list: parameter_list.into_iter().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::extend_parameter_list;

    #[test]
    fn basic() {
        let input_attr = quote::quote!(.., mut input: i32, output: &mut i32);
        let input_item = quote::quote! {
            fn abs() {
                input = input.abs();
                *output = input;
            }
        };

        let expected = quote::quote! {
            fn abs(mut input: i32, output: &mut i32) {
                input = input.abs();
                *output = input;
            }
        };

        let actual = extend_parameter_list(input_attr, input_item);

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn as_is() {
        let input_attr = quote::quote!(..);
        let input_item = quote::quote! {
            fn empty() {}
        };

        let expected = quote::quote! {
            fn empty() {}
        };

        let actual = extend_parameter_list(input_attr, input_item);

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
