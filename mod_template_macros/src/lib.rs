mod attributes;
mod helper_attributes;
mod utils;

/// This attribute defines a function-like macro based on the template module it
/// annotated. One can define actual modules with variations by calling the
/// defined macro with different arguments.
///
/// # Example
///
/// To define a module-defining macro that called `define_foo_mod`, and let the
/// macro user to customize some functions in the module by specifying ways to
/// construct variables (`BAR`), substitute attributes (along with extend
/// the function signature) (`BAZ`), one can write:
///
/// ```ignore
/// #[mod_template::define(define_foo_mod; constructions(BAR), attribute_substitutions(BAZ))]
/// mod __ {
///     #[__CONSTRUCT(bar as BAR)]
///     #[__SUBSTITUTE(BAZ)]
///     fn an_fn() {
///         bar.do_something();
///     }
/// }
/// ```
///
/// Call the defined macro to produce an actual module (named as `actual_foo`):
/// ```ignore
/// define_foo_mod! {
///     mod actual_foo;
///     constructions {
///         BAR => crate::Bar::new(),
///     },
///     attribute_substitutions {
///         BAZ => #[baz::baz],
///         // If one want to extend the signature of the function annotated by
///         // `#[__SUBSTITUTE(BAZ)]`, use
///         // `BAZ => #[baz::baz] (.., qux: crate::Qux),`.
///     },
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn define(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    attributes::define(attr.into(), item.into()).into()
}

/// This attribute is used by the attribute `mod_template::define` internally.
/// One should not use this attribute manually.
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn __monomorphize_mod(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    attributes::__monomorphize_mod(attr.into(), item.into()).into()
}

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
    attributes::construct(attr.into(), item.into()).into()
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
    attributes::extend_parameter_list(attr.into(), item.into()).into()
}
