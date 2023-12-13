fn main() {}

trait Sumer {
    fn sum(&self, iter: Box<dyn Iterator<Item = i32>>) -> i32;
}

struct ImperativeSumer;
impl Sumer for ImperativeSumer {
    fn sum(&self, iter: Box<dyn Iterator<Item = i32>>) -> i32 {
        let mut sum = 0;
        for item in iter {
            sum += item
        }
        sum
    }
}

struct FunctionalSumer;
impl Sumer for FunctionalSumer {
    fn sum(&self, iter: Box<dyn Iterator<Item = i32>>) -> i32 {
        iter.reduce(|a, b| a + b).unwrap_or(0)
    }
}

struct BuiltinSumer;
impl Sumer for BuiltinSumer {
    fn sum(&self, iter: Box<dyn Iterator<Item = i32>>) -> i32 {
        iter.sum()
    }
}

#[mod_template::define(define_sumer_test_suite; constructions(SUMER), attribute_substitutions(TEST))]
mod __ {
    // FIXME: rust thinks that the above import is unused during `cargo test`,
    // why? (My editor's lsp, `cargo check`, and `cargo clippy` still work
    // fine in this case.)
    #[allow(unused_imports)]
    use crate::Sumer;

    #[__CONSTRUCT(sumer as SUMER)]
    #[__SUBSTITUTE(TEST)]
    fn it_works() {
        let input = vec![1, 2, 3];
        assert_eq!(sumer.sum(Box::new(input.into_iter())), 6)
    }
}

define_sumer_test_suite! {
    mod imperative_sumer_test_suite;
    constructions {
        SUMER => crate::ImperativeSumer{},
    },
    attribute_substitutions {
        TEST => #[test],
    },
}

define_sumer_test_suite! {
    mod functional_sumer_test_suite;
    constructions {
        SUMER => crate::FunctionalSumer{},
    },
    attribute_substitutions {
        TEST => #[test],
    },
}

define_sumer_test_suite! {
    mod builtin_sumer_test_suite;
    constructions {
        SUMER => crate::BuiltinSumer{}
    },
    attribute_substitutions {
        TEST => #[test],
    },
}
