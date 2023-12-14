/// This attribute defines a function-like macro based on the template module it
/// annotated. One can define actual modules with variations by calling the
/// defined macro with different arguments.
///
/// # Example
///
/// ```ignore
/// // To define a module-defining macro that called `define_foo_mod`, and let
/// // the macro user to customize some functions in the module by specifying
/// // ways to construct variables (`BAR`), substitute attributes (along with
/// // extend the function signature) (`BAZ`), one can write:
/// #[mod_template::define(
///     define_foo_mod;
///     constructions(BAR -> impl crate::Bar),
///     attribute_substitutions(BAZ)),
/// ]
/// mod __ {
///     #[__CONSTRUCT(bar as BAR)]
///     #[__SUBSTITUTE(BAZ)]
///     fn an_fn() {
///         bar.do_something();
///     }
/// }
///
/// // Call the defined macro to produce an actual module (named as `actual_foo`):
/// define_foo_mod! {
///     mod actual_foo;
///     constructions {
///         BAR => crate::Bar::new(),
///     },
///     attribute_substitutions {
///         BAZ => #[::baz::baz],
///         // If one want to extend the signature of the function annotated by
///         // `#[__SUBSTITUTE(BAZ)]`, use
///         // `BAZ => #[::baz::baz] (.., qux: crate::Qux),`.
///     },
/// }
/// ```
pub use mod_template_macros::define;

/// This attribute is used by the attribute `mod_template::define` internally.
/// One should not use this attribute manually.
pub use mod_template_macros::__monomorphize_mod;

/// Turns something like:
///
/// ```
/// #[mod_template::construct(one = 1, mut to_be_three: i32 = 2)]
/// #[test]
/// fn test_one_adds_three() {
///     to_be_three += 1;
///     assert_eq!(format!("{}", one + to_be_three), four_text.to_string())
/// }
/// ```
///
/// into:
///
/// ```no_run
/// #[test]
/// fn test_one_adds_three() {
///     let one = 1;
///     let mut to_be_three: i32 = 2;
///     let four_text = {
///         fn type_checked() -> impl std::fmt::Display { "4" }
///         type_checked()
///     };
///     to_be_three += 1;
///     assert_eq!(format!("{}", one + to_be_three), four_text.to_string())
/// }
/// ```
pub use mod_template_macros::construct;

/// Turns something like:
///
/// ```
/// #[mod_template::extend_parameter_list(.., mut input: i32, output: &mut i32)]
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
pub use mod_template_macros::extend_parameter_list;
