use std::cell::RefCell;
use std::io;
use std::path::Path;
use std::rc::Rc;
use indexmap::IndexMap;
use crate::cache::temp_file_manager::TempFileManager;
use crate::mapped_file::MappedFile;
use std::io::Result;
static MANAGER_PATH: &str = "/tmp/tmpfs/cache";

/// File cache that maps the path of the file to a rc<mmap> smart pointer
#[derive(Debug)]
pub struct TempFileCache {
    capacity: usize,
    size: usize,
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
            size: 0,
            ordered_map: IndexMap::new(),
            file_manager: TempFileManager::new(Path::new(&path)),
        }
    }

    /// Creates a new temporary file that is mapped on the given string.
    /// Evicts the least used file from the cache if at full capacity.
    /// If the key already exists nothing changes.
    pub fn insert(&mut self,key: String){

        if let Some(_) = self.peek(&key){
            return;
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

    /// Writes the data buffer to the chosen file if it exists.
    /// Evicts least recently used files while the cache cannot hold the new capacity.
    /// Affects the map state as the file that is written into is not supposed to be evicted in case the cache is full.
    pub fn write_append(&mut self,key: &str,data: &[u8]) -> Result<()>{

        let data_size = data.len();

        if data_size > self.capacity {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Exceeded capacity"));
        }

        // check if the file exists
        let mut mapped_file = match self.get(&key){
            Some(mapped_file) => mapped_file,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "Cache file key not found"))
        };

        // start evicting if the cache is passed the full capacity
        while data_size + self.size > self.capacity{

            // obtain the first key
            let (key,mapped_file) = self.ordered_map.first().unwrap();

            // get the evicted file size
            let mmap_ptr = mapped_file.borrow();
            let evicted_file_size = mmap_ptr.file_size();

            // evict the temporary file and decrement the current cache size
            self.file_manager.evict(key).expect("Key eviction error");
            self.size -= evicted_file_size;

            // drop so that the borrow checker will not yell at me
            drop(mmap_ptr);
            // remove oldest entry
            self.ordered_map.shift_remove_index(0);
        }

        // write to file and update the size
        let mut mmap_ptr = mapped_file.borrow_mut();
        mmap_ptr.write_append(data)?;

        self.size += data_size;

        Ok(())

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

        if let Some(_) = self.ordered_map.get(key){
            return Some(Rc::clone(self.ordered_map.get(key).unwrap()));
        }

        None
    }

}

