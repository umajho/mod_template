use flex_mod::{construct, extend_parameter_list};

fn main() {}

#[construct(two = 2)]
#[extend_parameter_list(.., addend: i32)]
fn add_two_a() -> i32 {
    addend + two
}

#[extend_parameter_list(.., addend: i32)]
#[construct(two = 2)]
fn add_two_b() -> i32 {
    addend + two
}

#[test]
fn test_add_two_a() {
    assert_eq!(add_two_a(1), 3);
}

#[test]
fn test_add_two_b() {
    assert_eq!(add_two_b(1), 3);
}
