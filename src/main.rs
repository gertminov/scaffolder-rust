use std::io;
//pub mod backend;

pub mod front_end;
fn main() -> io::Result<()>{
    front_end::ui::init_ui()
}

