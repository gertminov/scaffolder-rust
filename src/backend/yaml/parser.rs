use std::string::ToString;
use serde::Deserialize;
use serde_yaml::{Mapping, Sequence, Value};
use slab_tree::TreeBuilder;

enum NodeType {
    Children{name: String},
    SingleSelect{options: [String], choice: String},
    MultiSelect{options: [String], choices: [String]}
}
impl NodeType {
    pub const CHILDREN: String = String::from("children");
    pub const OPTIONS: String = String::from("options");
}
pub fn parse_project_yaml(yaml_str: &str){
    let de = serde_yaml::Deserializer::from_str(yaml_str);
    let dings = Value::deserialize(de).expect("error while deserialzing");
    let mapping = dings.as_mapping().expect("No Mapping");
    let project = mapping.get("project").map(|p| p.as_mapping()).flatten().expect("No Project");
    let default_location = project.get("default_location").map(|l| l.as_str()).flatten();
    TreeBuilder::new().with_root(default_location).build()
    walk_project(project);
}

fn walk_project(project: &Mapping){
    println!("{:?}", project);
    let children = project.get(NodeType::CHILDREN).map(|c| c.as_sequence()).flatten();
    match children {
        None => {return;}
        Some(cs) => {}
    }
}