use std::{error::Error, io};
use slab_tree::Tree;
use crate::backend;
use backend::tree::nodes::LeafNodeType;
mod ui;

pub fn init_ui(tree: Tree<LeafNodeType>) -> io::Result<()>{
    ui::init_ui(tree)
}
