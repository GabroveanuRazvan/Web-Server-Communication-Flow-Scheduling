use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use indexmap::IndexMap;
use memmap2::MmapMut;
use crate::temp_file_manager::TempFileManager;
use std::fs::File;
use std::io;
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
        let mut mapped_file = match MappedFile::new(file){
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

/// Data structure used to store a file and its mapped content
#[derive(Debug)]
pub struct MappedFile{

    file: File,
    mmap: MmapMut,

}

impl MappedFile{
    /// Create a new map from a given file
    pub fn new(file: File) -> io::Result<Self>{

        let mmap = unsafe{MmapMut::map_mut(&file)?};

        Ok(Self{
            file,
            mmap,
        })

    }

    /// Used to flush the written data on the disk
    pub fn flush(&mut self) -> io::Result<()>{
        self.mmap.flush()
    }

    /// Writes the new data at the end of the file
    pub fn write_append(&mut self,data: &[u8]) -> io::Result<()>{

        let current_file_size = self.file.metadata().unwrap().len() as usize;
        let new_size = current_file_size + data.len();

        self.file.set_len(new_size as u64).unwrap();

        self.mmap = unsafe{MmapMut::map_mut(&self.file)?};

        self.mmap[current_file_size..new_size].copy_from_slice(data);

        Ok(())
    }

    /// Getter for the raw slice of bytes of the file
    pub fn mmap_as_slice(&self) -> &[u8]{
        &self.mmap
    }

}