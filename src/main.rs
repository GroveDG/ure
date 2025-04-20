use sys::{tf::Space2D, tree::Tree, UIDs};

mod sys;

fn main() {
    let mut uids: UIDs = UIDs::new().expect("UID RNG failed to intitialize");
    let mut tree: Tree = Tree::new(uids.new_uid());
    let mut transform: Space2D = Default::default();

    loop {
        
    }
}
