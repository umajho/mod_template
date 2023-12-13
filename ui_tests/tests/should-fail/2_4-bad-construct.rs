fn main() {}

#[mod_template::define(define_foo; constructions(CONS))]
mod __ {
    #[__CONSTRUCT(CONS)]
    fn an_fn() {}
}
