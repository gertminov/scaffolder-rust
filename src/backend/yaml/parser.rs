use std::fmt::Debug;
use std::string::ToString;
use crossterm::event::Event::Key;
use serde::de::Unexpected::Str;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Sequence, Value};
use slab_tree::*;
use slab_tree::NodeMut;

enum NodeType {
    Child{name: String},
    Option{options: Vec<String>, name: String },
    TextInput{input: String, name: String}
}

impl std::fmt::Debug for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Child{name} => write!(f, "{}", name),
            NodeType::Option{options, name} => {
                let options_as_string: String = options.join(", ");
                write!(f, "{} with [{}]", name, options_as_string)
            },
            NodeType::TextInput {input, name} => write!(f, "{}: {}", name, input ),
            // Add formatting for additional variants
        }
    }
}
pub fn parse_project_yaml(yaml_str: &str){
    let de = serde_yaml::Deserializer::from_str(yaml_str);
    let value = Value::deserialize(de).expect("error while deserialzing");
    let mapping = value.as_mapping().expect("No Mapping");
    let project = mapping.get("project").map(|p| p.as_mapping()).flatten().expect("No Project");
    let default_location = project.get("default_location").map(|l| l.as_str()).flatten().unwrap_or("~/");
    let root_node = NodeType::TextInput {name: String::from("Location"), input: String::from(default_location)};
    let mut tree = TreeBuilder::new().with_root(root_node).build();
    println!("{:?}\n", project);
    walk_project(project,&mut tree.root_mut().unwrap());

    let mut s = String::new();
    tree.write_formatted(&mut s).unwrap();
    println!("{}", s);
}

fn walk_project(project: &Mapping, root: &mut NodeMut<NodeType>) {
    //println!("{:?}\n", project);
    let children = project.get("children");
    let options = project.get("options");
    let child_options = project.get("childoptions");
    match child_options{
        Some(childs) => {
            for child in childs.as_sequence().unwrap() {
                if let Some(child_as_mapping) = child.as_mapping() {
                    let keys = child_as_mapping.keys();
                    for key in keys {
                        if let Some(key_str) = key.as_str() {
                            let mut node = root.append(NodeType::Child {name: String::from(key_str)});
                            if let Some(value_of_child_as_mapping) = child_as_mapping.get(key_str).unwrap().as_mapping() {
                                walk_project(value_of_child_as_mapping, &mut node);
                            }
                        }
                    }
                }
                else if let Some(sequence_as_string) = child.as_str() {
                    root.append(NodeType::Child {name: String::from(sequence_as_string)});
                }
            }
        }
        None => {}
    }
    match children{
       Some(childs) => {
            for child in childs.as_sequence().unwrap() {
                if let Some(child_as_mapping) = child.as_mapping() {
                    let key_opt = child_as_mapping.keys().next().map(|key| key.as_str()).flatten();
                    if let Some(key) = key_opt {
                        let mut node = root.append(NodeType::Child {name: String::from(key)});
                        if let Some(value_of_child_as_mapping) = child_as_mapping.get(key).unwrap().as_mapping() {
                            walk_project(value_of_child_as_mapping, &mut node);
                        }
                    }
                }
                else if let Some(sequence_as_string) = child.as_str() {
                    root.append(NodeType::Child {name: String::from(sequence_as_string)});
                }
            }
        }
        None => {}
    }
    match options{
        Some(opts) => {
            /*let opt_vec = opts.as_sequence().expect("found options key but no options")
                .map(|entry| entry.as_str()).flatten();*/
        }
        None => {}
    }
}