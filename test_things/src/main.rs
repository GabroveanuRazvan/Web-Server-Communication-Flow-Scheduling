use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Component, Components, Path, PathBuf};

fn main() {

    let mut path = Path::new("/web/dir1/file1.txt");
    let parh2 = Path::new("/web/dir1/file2.txt");
    let mut trie = Trie::new();

    trie.insert(path);
    trie.insert(parh2);

    println!("{:?}",trie);

}

#[derive(Debug)]
struct TrieNode {
    current_dir: Box<String>,
    children: HashMap<Box<String>,TrieNode>,
    is_file: bool,
}

impl TrieNode{
    fn new(current_dir: Box<String>) -> Self{
        Self{
            current_dir,
            children: HashMap::new(),
            is_file: false,
        }
    }
}

#[derive(Debug)]
struct Trie{
    root: TrieNode,
}

impl Trie{
    pub fn new() ->Self{
        Self{
            root: TrieNode::new(Box::new("/".to_string())),
        }
    }

    pub fn insert(&mut self, path: &Path){

        let mut current_node = &mut self.root;

        for chunk in path.components(){

            if let Component::Normal(dir) = chunk{

                let dir = Box::new(dir.to_string_lossy().into_owned());
                let entry = current_node.children.entry(dir.clone()).or_insert(TrieNode::new(dir.clone()));
                current_node = current_node.children.get_mut(&dir).unwrap();

            }

        }

        current_node.is_file = true;

    }

    pub fn find(&mut self,path: &Path) -> bool{

        let mut current_node = &mut self.root;

        for chunk in path.components(){
            if let Component::Normal(dir) = chunk{
                let dir = Box::new(dir.to_string_lossy().into_owned());

                match current_node.children.get_mut(&dir){
                    None => return false,
                    Some(node) => current_node = node,
                }

            }
        }

        true

    }
}
