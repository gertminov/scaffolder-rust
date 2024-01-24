use std::fs;
use user_error::{UFE, UserFacingError};

pub fn read_file(file_path: &str)-> String {
     return fs::read_to_string(file_path).map_err(|_| {
         UserFacingError::new("Could not read template.yaml file")
             .reason("file might be missing")
             .help("a bare bones template file will be created\
                 in the directory of the scaffolder executable")
             .print();
         let new_template_path = std::env::current_dir()
             .expect("exe ist not in a dir... wtf??").join("template.yaml");
         fs::write(&new_template_path, "project:\n  default_location: \"~/my_project/\"")
             .expect("could not write template file.\
                 \n Please create a file called template.yaml in the directory of the scaffolder executable.\
                  \n You can copy the following content into the file:\
                   \n project:\n  default_location: \"~/my_project/\"");
            std::process::exit(1);

     }).unwrap();
}