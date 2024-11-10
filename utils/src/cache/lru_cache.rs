use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use indexmap::IndexMap;
use crate::cache::temp_file_manager::TempFileManager;
use crate::mapped_file::MappedFile;

static MANAGER_PATH: &str = "/tmp/cache";

/// File cache that maps the path of the file to a rc<mmap> smart pointer
#[derive(Debug)]
pub struct TempFileCache {
    capacity: usize,
    ordered_map: IndexMap<String, Rc<RefCell<MappedFile>>>,
    file_manager: TempFileManager,
}


impl TempFileCache {

    /// Creates an empty cache with a non-negative capacity
    pub fn new(capacity: usize) -> Self{
        assert!(capacity > 0);

        // get a unique name for every cache
        let path = String::from(MANAGER_PATH) + TempFileManager::unique_name().as_str();

        Self{
            capacity,
            ordered_map: IndexMap::new(),
            file_manager: TempFileManager::new(Path::new(&path)),
        }
    }

    /// Creates a new temporary file that is mapped on the given string.
    /// Evicts the least used file from the cache if at full capacity.
    /// If the key already exists nothing changes
    pub fn insert(&mut self,key: String){

        if let Some(_) = self.peek(&key){
            return;
        }

        let map_size = self.ordered_map.len();

        if map_size == self.capacity{

            // obtain the first key
            let (key,_) = self.ordered_map.first().unwrap();
            // evict the temporary file
            self.file_manager.evict(key);

            // remove oldest entry
            self.ordered_map.shift_remove_index(0);

        }

        // open and create a new temporary file
        let file = match self.file_manager.open(key.clone()){
            Ok(file) => file,
            Err(error) => panic!("Error while inserting a new file into the cache: {}",error)
        };

        // using the created and opened file create a memory map to it
        let mapped_file = match MappedFile::new(file){
            Ok(mapped_file) => mapped_file,
            Err(error) => panic!("Error while creating a mapped file: {}",error)
        };

        // finally insert the key and the mapped file
        self.ordered_map.insert(key, Rc::new(RefCell::new(mapped_file)));

    }

    /// Obtains the wrapped MappedFile from the cache if the cache is hit
    pub fn get(&mut self,key: &str)-> Option<Rc<RefCell<MappedFile>>>{

        if let Some(value) = self.ordered_map.get(key){

            // get a copy of the value
            let value_copy = Rc::clone(value);
            // remove the old entry; value goes out of scope
            self.ordered_map.shift_remove(key);
            // insert the got value
            self.ordered_map.insert(key.to_string(), value_copy);

            return Some(Rc::clone(self.ordered_map.get(key).unwrap()));

        }

        None
    }

    /// Obtains the wrapped MappedFile from the cache without changing cache state
    pub fn peek(&mut self,key: &str) -> Option<Rc<RefCell<MappedFile>>>{

        if let Some(value) = self.ordered_map.get(key){
            return Some(Rc::clone(self.ordered_map.get(key).unwrap()));
        }

        None
    }

}

