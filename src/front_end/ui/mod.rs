use std::{error::Error, io};
use crate::backend::tree::nodes::LeafNodeType;
use slab_tree::*;

pub(crate) mod ui;

pub fn init_ui(tree: Tree<LeafNodeType>) -> io::Result<()>{
    ui::init_ui(tree)
}
