use mod_template::{construct, extend_parameter_list};

fn main() {}

#[construct]
mod a_mod {}

#[extend_parameter_list(..)]
struct a_struct {}
