use std::fs;
use std::path::PathBuf;
use slab_tree::{NodeRef, Tree};

pub fn build_folder_structure(tree: Tree<String>) -> () {
    let root = tree.root().expect("Error, tree has no root");
    let dir_path = PathBuf::new();
    walk_tree(root, &dir_path);
}

fn walk_tree(node: NodeRef<String>, parent_path: &PathBuf){
    let node_name = node.data();
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
        let mut tree: Tree<String> = TreeBuilder::new()
            .with_root(String::from("root"))
            .build();
        let mut root = tree.root_mut().unwrap();
        append_children(2, &mut root);
        build_folder_structure(tree);
    }

    /*fn get_node(name: &str) -> LeafNodeType {
        LeafNodeType::Text { name: name.to_string() }
    }*/

    fn append_children(amt: u16, parent: &mut NodeMut<String>) {
        for i in 0..amt {
            let mut child = parent.append(format!("child{}", i+1));
            child.append(format!("sub_child{}", i+1));
        }

    }
}