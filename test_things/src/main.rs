use std::fs::create_dir_all;
use std::path::Path;

fn main() {
    
    let path = Path::new("mere");
    let parent = path.parent();
    create_dir_all("").unwrap();
    
    println!("{:?}", parent);
    
}