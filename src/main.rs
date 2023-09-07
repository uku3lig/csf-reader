use csf_reader::play::play_sync;
use csf_reader::CsfRoot;

fn main() {
    let root_path = std::env::args().nth(1).expect("no root path provided");
    let root = CsfRoot::new_eager(std::path::PathBuf::from(root_path)).unwrap();

    play_sync(root).unwrap();
}
