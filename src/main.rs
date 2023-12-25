use std;
use std::io;
use cli_clipboard;
pub mod backend;
pub mod front_end;

fn main() ->  Result<(), io::Error>{
    let tree = backend::yaml::parse_yaml("./structure.yaml");
    let ui_res = front_end::ui::init_ui(tree);

    match ui_res {
        Ok(tree_opt) => {
            if let Some(tree) = tree_opt {
                //cpy path to clipboard
                let project_path = tree.root().expect("Error, tree has no root").data();
                cli_clipboard::set_contents(project_path.to_owned()).unwrap();

                //build folder structure
                backend::folder::build_folder_structure(tree);
            }
            Ok(())
        }
        Err(e) => {
            return Err(e); }
    }
}

