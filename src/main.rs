use std;
use slab_tree::*;

pub mod backend;
pub mod front_end;
fn main() -> (){

    let tree = backend::yaml::parse_yaml("./structure.yaml");

    front_end::ui::init_ui(tree);
}

