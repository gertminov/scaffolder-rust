use std::fs;
use std::path::PathBuf;
use slab_tree::{NodeRef, Tree};
use crate::backend::tree::nodes::LeafNodeType;

fn build_folder_structure(tree: Tree<LeafNodeType>) -> () {
    let root = tree.root().expect("no root");
    let dir_path = PathBuf::new();
    walk_tree(root, &dir_path);
}

fn walk_tree(node: NodeRef<LeafNodeType>, parent_path: &PathBuf){
    let node_name = node.data().get_name();
    let dir_path = parent_path.join(node_name);
    create_folder(&dir_path);
    node.children().for_each(|child| {
        walk_tree(child, &dir_path)

    });
}

fn create_folder(name: &PathBuf) {
    fs::create_dir_all(name).expect(format!("could not create folder: {:?}", name).as_str())
}

#[cfg(test)]
mod tests {
    use slab_tree::{NodeMut, TreeBuilder};
    use super::*;

    #[test]
    fn test_build_folder_structure() {
        let mut tree = TreeBuilder::new()
            .with_root(get_node("root"))
            .build();
        let mut root = tree.root_mut().unwrap();
        append_children(2, &mut root);
        build_folder_structure(tree);
    }

    fn get_node(name: &str) -> LeafNodeType {
        LeafNodeType::Text { name: name.to_string() }
    }

    fn append_children(amt: u16, parent: &mut NodeMut<LeafNodeType>) {
        for i in 0..amt {
            let mut child = parent.append(get_node(format!("child{}", i+1).as_str()));
            child.append(get_node(format!("sub_child{}", i+1).as_str()));
        }

    }
}