use slab_tree::Tree;

mod builder;

pub fn build_folder_structure(tree: Tree<String>) -> () {
    builder::build_folder_structure(tree);
}