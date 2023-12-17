use std;
pub mod backend;

pub mod front_end;
fn main() {

    let mut tree = backend::yaml::parse_yaml("./structure.yaml");

    let mut s = String::new();
    tree.write_formatted(&mut s).unwrap();
    println!("{}", s);
    front_end::ui::init_ui(tree);
}

