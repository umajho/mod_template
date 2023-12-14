use mod_template::construct;

fn main() {}

#[construct(
    one = 1, mut to_be_three: i32 = 2,
    four_text: impl std::fmt::Display = "4",
)]
#[test]
fn test_one_adds_three() {
    to_be_three += 1;
    assert_eq!(format!("{}", one + to_be_three), four_text.to_string())
}

#[construct(
    one = 1, mut to_be_three: i32 = 2,
    four_text: impl std::fmt::Display = "4",
)]
async fn async_assert_one_adds_three() {
    to_be_three += 1;
    assert_eq!(format!("{}", one + to_be_three), four_text.to_string())
}

#[construct(
    one = 1, mut to_be_three: i32 = 2,
    four_text: impl std::fmt::Display = format!("{}", four),
)]
fn assert_one_adds_three_with_external_four(four: i32) {
    to_be_three += 1;
    assert_eq!(format!("{}", one + to_be_three), four_text.to_string())
}
