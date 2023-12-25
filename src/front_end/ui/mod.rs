use std::io;
use crate::backend::tree::nodes::LeafNodeType;
use slab_tree::*;

pub(crate) mod ui;

pub fn init_ui(tree: Tree<LeafNodeType>) -> Result<Option<Tree<String>>, io::Error>{
    ui::init_ui(tree)
}
