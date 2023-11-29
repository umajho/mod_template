/// Turns something like:
///
/// ```ignore
/// #[construct(one = 1, mut to_be_three = 2)]
/// fn four() -> i32 {
///     to_be_three += 1;
///     one + to_be_three
/// }
/// ```
///
/// into:
///
/// ```no_run
/// fn four() -> i32 {
///     let one = 1;
///     let mut to_be_three = 2;
///     to_be_three += 1;
///     one + to_be_three
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn construct(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    flex_mod_macros_core::construct(attr.into(), item.into()).into()
}

/// Turns something like:
///
/// ```ignore
/// #[extend_parameter_list(.., mut input: i32, output: &mut i32)]
/// fn abs() {
///     input = input.abs();
///     *output = input;
/// }
/// ```
///
/// into:
///
/// ```no_run
/// fn abs(mut input: i32, output: &mut i32) {
///     input = input.abs();
///     *output = input;
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn extend_parameter_list(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    flex_mod_macros_core::extend_parameter_list(attr.into(), item.into()).into()
}
