use std::fmt::{Debug, Display};
use slab_tree::{NodeId, NodeMut, NodeRef, Tree};
use crate::front_end::ui::ui::StatefulList;

pub enum LeafNodeType {
    Text { name: String },
    Option { options: StatefulList, name: String },
    TextInput { name: String, input: String },
}

pub trait CloneTree {
    fn clone(&self) -> Tree<LeafNodeType>;
}
impl CloneTree for Tree<LeafNodeType> {
    fn clone(&self) -> Tree<LeafNodeType>{
        let root = self.root().expect("Tree has no root");
        let mut cloned_tree: Tree<LeafNodeType> = Tree::new();
        cloned_tree.set_root(root.data().clone());

        fn clone_node(node: NodeRef<LeafNodeType>, mut cloned_node: NodeMut<LeafNodeType>) {
            for child in node.children() {
                let cloned_child = cloned_node.append(child.data().clone());
                clone_node(child, cloned_child);
            }
        }
        clone_node(root ,cloned_tree.root_mut().expect("preview tree generation failed"));

        cloned_tree
    }
}

pub trait NodeIndex{
    fn node_index(&self, node_id: NodeId) -> usize;
}

impl NodeIndex for Tree<LeafNodeType> {
    fn node_index(&self, node_id: NodeId) -> usize {
        fn index(node: NodeRef<LeafNodeType>, vertical_index: &mut usize, node_id: NodeId, recursive_break:  &mut bool) {

            for child in node.children() {
                if child.node_id() == node_id{
                    *recursive_break = true;
                    *vertical_index += 1;
                    return;
                } else if *recursive_break { return; }
                *vertical_index += 1;
                index(child, vertical_index, node_id, recursive_break);
            }
        }
        let mut recursive_break = false;
        let mut vertical_index = 0;
        let root = self.root().expect("indexing of tree node failed, Tree has no root");
        if root.node_id() != node_id {
            index(root, &mut vertical_index, node_id, &mut recursive_break);
        }

        vertical_index
    }
}

impl LeafNodeType {
    pub fn get_name(&self) -> &str {
        match self {
            LeafNodeType::Text { name } => { name }
            LeafNodeType::Option { name, .. } => { name }
            LeafNodeType::TextInput { name, .. } => { name }
        }
    }
    pub fn clone(&self) -> LeafNodeType {
        match self {
            LeafNodeType::Option {name, options} => LeafNodeType::Option {name: name.clone(), options: options.clone()},
            LeafNodeType::TextInput {name, input} => LeafNodeType::TextInput{name: name.clone(), input: input.clone()},
            LeafNodeType::Text {name} => return LeafNodeType::Text {name: name.clone()},
        }
    }
}


impl Debug for LeafNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeafNodeType::Text { name } => write!(f, "{}", name),
            LeafNodeType::Option { name, options } => {
                let options_as_string: String = options.join_names_with(", ");
                write!(f, "{:?} with [{}]", name, options_as_string)
            }
            LeafNodeType::TextInput { name, input } => write!(f, "{:?} input: {}", name, input),
            // Add formatting for additional variants
        }
    }
}

impl Display for LeafNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeafNodeType::Text { name } => write!(f, "{}", name),
            LeafNodeType::Option { name, options } => {
                let options_as_string: String = options.join_names_with(", ");
                write!(f, "{:?} with [{}]", name, options_as_string)
            }
            LeafNodeType::TextInput { name, input } => write!(f, "{:?} input: {}", name, input),
            // Add formatting for additional variants
        }
    }
}
