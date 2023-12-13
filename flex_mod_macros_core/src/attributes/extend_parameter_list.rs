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

pub struct AttributeOptions {
    direction: Direction,
    parameter_list: Vec<TokenTree>,
}

impl AttributeOptions {
    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn parameter_list(&self) -> &Vec<TokenTree> {
        &self.parameter_list
    }

    pub fn is_noop(&self) -> bool {
        self.parameter_list.is_empty()
    }
}

/// Only `append` is supported for now
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
pub enum Direction {
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

impl quote::ToTokens for AttributeOptions {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.direction() {
            Direction::Append => quote::quote!(..,).to_tokens(tokens),
        }
        let list = self.parameter_list().clone();
        TokenStream::from_iter(list).to_tokens(tokens);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use proc_macro2::TokenStream;

    use super::{extend_parameter_list, AttributeOptions, Direction};

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

    #[derive(Debug, PartialEq, Eq, typed_builder::TypedBuilder)]
    pub struct AttributeOptionsForTest {
        direction: Direction,
        parameter_list: String,
    }
    impl From<AttributeOptions> for AttributeOptionsForTest {
        fn from(value: AttributeOptions) -> Self {
            let AttributeOptions {
                direction,
                parameter_list,
            } = value;

            Self {
                direction,
                parameter_list: TokenStream::from_iter(parameter_list).to_string(),
            }
        }
    }
}
