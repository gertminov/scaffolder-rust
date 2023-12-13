use std::fmt::Debug;

pub enum LeafNodeType {
    Text { name: String },
    Option { options: Vec<String>, name: String },
    TextInput { name: String, input: String },
}

impl LeafNodeType {
    fn get_name(&self) -> &str {
        match self {
            LeafNodeType::Text { name } => { name }
            LeafNodeType::Option { name, .. } => { name }
            LeafNodeType::TextInput { name, .. } => { name }
        }
    }
}


impl Debug for LeafNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeafNodeType::Text { name } => write!(f, "{}", name),
            LeafNodeType::Option { name, options } => {
                let options_as_string: String = options.join(", ");
                write!(f, "{:?} with [{}]", name, options_as_string)
            }
            LeafNodeType::TextInput { name, input } => write!(f, "{:?} input: {}", name, input),
            // Add formatting for additional variants
        }
    }
}
