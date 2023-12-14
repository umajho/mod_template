mod attributes;
mod helper_attributes;
mod utils;

/// See [`mod_template::define`](../mod_template/attr.define.html).
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn define(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    attributes::define(attr.into(), item.into()).into()
}

/// See [`mod_template::__monomorphize_mod`](../mod_template/attr.__monomorphize_mod.html).
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn __monomorphize_mod(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    attributes::__monomorphize_mod(attr.into(), item.into()).into()
}

/// See [`mod_template::construct`](../mod_template/attr.construct.html).
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn construct(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    attributes::construct(attr.into(), item.into()).into()
}

/// See [`mod_template::extend_parameter_list`](../mod_template/attr.extend_parameter_list.html).
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn extend_parameter_list(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    attributes::extend_parameter_list(attr.into(), item.into()).into()
}
