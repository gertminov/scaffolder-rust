use std::fmt::Debug;

use serde::Deserialize;
use serde_yaml::{Mapping, Sequence, Value};
use slab_tree::*;
use slab_tree::NodeMut;


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

pub fn parse_project_yaml(yaml_str: &str) {
    let de = serde_yaml::Deserializer::from_str(yaml_str);
    let value = Value::deserialize(de).expect("error while deserialzing");
    let mapping = value.as_mapping().expect("No Mapping");
    let project = mapping.get("project").map(|p| p.as_mapping()).flatten().expect("No Project");
    let default_location = project.get("default_location").map(|l| l.as_str()).flatten().unwrap_or("~/");
    let root_node = LeafNodeType::TextInput {
        name: "Location".to_string(),
        input: default_location.to_string(),
    };
    let mut tree = TreeBuilder::new().with_root(root_node).build();
    walk_project(project, &mut tree.root_mut().unwrap());

    let mut s = String::new();
    tree.write_formatted(&mut s).unwrap();
    println!("{}", s);
}

fn visit_children(children: &Sequence, parent: &mut NodeMut<LeafNodeType>, child_options: Option<Vec<String>>) {
    for child in children {
        if let Some(child_as_map) = child.as_mapping() {
            let (node_name, children) = child_as_map.iter().next()
                .expect("Warum kein child. WTF");
            if let Some(key) = node_name.as_str() {
                if let Some(value_of_child_as_mapping) = children.as_mapping() {
                    let node_type = get_node_type(value_of_child_as_mapping, key, &child_options);
                    let mut node = parent.append(node_type);
                    walk_project(value_of_child_as_mapping, &mut node);
                }
            }
        } else if let Some(leaf) = child.as_str() {
            if let Some(opts) = &child_options {
                let node_type = get_node_type(&Mapping::new(), leaf, &Some(opts.clone()));
                parent.append(node_type);
            } else {
                parent.append(LeafNodeType::Text { name: leaf.to_string() });
            }
        }
    }
}

fn visit_options(options: &Sequence) -> Vec<String> {
    options.iter().map(|o|
        o.as_str().expect(format!("could not read option: {:?}", o).as_str()).to_string()
    )
        .collect()
}

fn get_node_type(project: &Mapping, name: &str, child_options: &Option<Vec<String>>) -> LeafNodeType {
    let options = project.get("options");
    let seq_options = options
        .map(|o| o.as_sequence())
        .flatten()
        .map(|s| visit_options(s));

    let all_options = match (seq_options, child_options) {
        (None, None) => { None }
        (Some(opt_list), None) => { Some(opt_list) }
        (None, Some(child_opts)) => { Some(child_opts.iter().map(|s| s.clone()).collect()) }
        (Some(opt_list), Some(child_opts)) => { Some(child_opts.iter().map(|s| s.clone()).chain(opt_list).collect()) }
    };


    return if let Some(opt_list) = all_options {
        LeafNodeType::Option {
            options: opt_list,
            name: name.to_string(),
        }
    } else if let Some(opt_str) = options.map(|o| o.as_str()).flatten() {
        LeafNodeType::TextInput { name: name.to_string(), input: opt_str.to_string() }
    } else {
        LeafNodeType::Text { name: name.to_string() }
    };
}

fn walk_project(project: &Mapping, parent: &mut NodeMut<LeafNodeType>) {
    let children_opt = project.get("children").map(|c| c.as_sequence()).flatten();
    let child_options = project.get("childoptions")
        .map(|o| o.as_sequence())
        .flatten()
        .map(|s| visit_options(s));

    if let Some(children) = children_opt {
        visit_children(children, parent, child_options);
    }
}