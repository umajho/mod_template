use flex_mod::flex_mod;

fn main() {}

#[flex_mod(define_foo; constructions(CONS))]
mod __ {
    #[__CONSTRUCT(CONS)]
    fn an_fn() {}
}
