use slab_tree::Tree;
use crate::backend::tree::nodes::LeafNodeType;
use crate::backend::yaml::parser::parse_project_yaml;
use crate::backend::yaml::reader::read_file;

mod parser;
mod reader;

pub fn parse_yaml(path: &str) -> Tree<LeafNodeType> {
    let yaml_str = read_file(path);
    parse_project_yaml(yaml_str.as_str())
}
