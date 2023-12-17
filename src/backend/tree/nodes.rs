use std::fmt::Debug;
use crate::front_end::ui::ui::StatefulList;

pub enum LeafNodeType {
    Text { name: String },
    Option { options: StatefulList, name: String },
    TextInput { name: String, input: String },
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
