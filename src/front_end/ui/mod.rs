use std::io;

mod ui;

pub fn init_ui() -> io::Result<()>{
    ui::init_ui()
}
