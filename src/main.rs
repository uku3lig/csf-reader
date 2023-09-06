use csf_reader::play::play;
use csf_reader::CsfRoot;

fn main() {
    let root_path = std::env::args().nth(1).expect("missing root path");
    let root = CsfRoot::new_eager(std::path::PathBuf::from(root_path)).unwrap();

    play(&root).unwrap();
}
