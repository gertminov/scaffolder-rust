pub mod backend;
pub mod front_end;
fn main() {
    backend::yaml::parse_yaml("./structure.yaml");
}
