use std::rc::Rc;
use std::sync::Arc;
use indexmap::IndexMap;
use memmap2::Mmap;


/// File cache that maps the path of the file to a rc<mmap> smart pointer
#[derive(Debug)]
pub struct FileCache{
    capacity: usize,
    map: IndexMap<String, Rc<Mmap>>,
}


impl FileCache{

    /// Creates an empty cache with a non-negative capacity
    pub fn new(capacity: usize) -> Self{
        assert!(capacity > 0);

        Self{
            capacity,
            map: IndexMap::new(),
        }
    }

    /// Inserts a new file into the cache
    pub fn insert(&mut self,key: String , value: Mmap){

        let map_size = self.map.len();

        if(map_size == self.capacity){
            // remove oldest entry
            self.map.shift_remove_index(0);

        }

        self.map.insert(key,Rc::new(value));

    }

    /// Obtains the file from the cache if the cache is hit
    pub fn get(&mut self,key: &str)-> Option<Rc<Mmap>>{

        if let Some(value) = self.map.get(key){

            // get a copy of the value
            let value_copy = Rc::clone(value);
            // remove the old entry; value goes out of scope
            self.map.shift_remove(key);
            // insert the got value
            self.map.insert(key.to_string(),value_copy);

            return Some(Rc::clone(self.map.get(key).unwrap()));

        }

        None
    }

}

