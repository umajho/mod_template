use mod_template::extend_parameter_list;

fn main() {}

#[extend_parameter_list(.., mut input: i32, output: &mut i32)]
fn abs() {
    input = input.abs();
    *output = input;
}

#[test]
fn test_abs() {
    let mut output = 0;
    abs(-42, &mut output);
    assert_eq!(output, 42);
}

#[extend_parameter_list(.., b: i32)]
fn add(a: i32) -> i32 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(1, 2), 3);
}
