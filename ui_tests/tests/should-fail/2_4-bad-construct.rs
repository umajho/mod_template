fn main() {}

#[mod_template::define(define_foo; constructions(CONS -> ToCons))]
mod __ {
    #[__CONSTRUCT(CONS)]
    fn an_fn() {}
}

#[mod_template::define(define_foo; constructions(CONS))]
mod __ {
    #[__CONSTRUCT(foo as CONS)]
    fn an_fn() {}
}
