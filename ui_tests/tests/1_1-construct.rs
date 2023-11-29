use flex_mod::construct;

fn main() {}

#[construct(one = 1, mut to_be_three = 2)]
#[test]
fn test_one_adds_three() {
    to_be_three += 1;
    assert_eq!(one + to_be_three, 4)
}
