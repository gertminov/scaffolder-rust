use std::fs;

pub fn read_file(file_path: &str)-> String {
    return fs::read_to_string(file_path).expect("could not read file");
}