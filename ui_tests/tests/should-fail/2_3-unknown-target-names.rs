fn main() {}

#[mod_template::define(macro_rules! define_foo; constructions(CONS -> ToCons), attribute_substitutions(ATTR_SUB))]
mod __ {
    #[__CONSTRUCT(foo as CONS)]
    fn good_construction() {}

    #[__SUBSTITUTE(ATTR_SUB)]
    fn good_attribute_substitution() {}

    #[__CONSTRUCT(foo as NOT_FOUND)]
    fn bad_construction_not_found() {}

    #[__SUBSTITUTE(NOT_FOUND)]
    fn bad_attribute_substitution_not_found() {}

    #[__CONSTRUCT(foo as ATTR_SUB)]
    fn bad_use_attribute_substitution_as_construction() {}

    #[__SUBSTITUTE(CONS)]
    fn bad_use_construction_as_attribute_substitution() {}
}
