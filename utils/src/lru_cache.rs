use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use indexmap::IndexMap;
use memmap2::{Mmap, MmapMut};
use crate::temp_file_manager;
use crate::temp_file_manager::TempFileManager;
use std::fs::File;
use std::io;
static MANAGER_PATH: &str = "/tmp/cache";

/// File cache that maps the path of the file to a rc<mmap> smart pointer
#[derive(Debug)]
pub struct FileCache{
    capacity: usize,
    map: IndexMap<String, Rc<Mmap>>,
    file_manager: TempFileManager,
}


impl FileCache{

    /// Creates an empty cache with a non-negative capacity
    pub fn new(capacity: usize) -> Self{
        assert!(capacity > 0);

        Self{
            capacity,
            map: IndexMap::new(),
            file_manager: TempFileManager::new(Path::new(MANAGER_PATH)),
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

pub struct MappedFile{

    pub file: File,
    pub mmap: MmapMut,

}

impl MappedFile{
    pub fn new(file: File) -> io::Result<Self>{

        let mmap = unsafe{MmapMut::map_mut(&file)?};

        Ok(Self{
            file,
            mmap,
        })

    }

    pub fn write(&mut self, data: &[u8]){
        self.mmap[..data.len()].copy_from_slice(data)
    }

    pub fn flush(&mut self) -> io::Result<()>{
        self.mmap.flush()
    }

    pub fn append(&mut self,data: &[u8]) -> io::Result<()>{

        let current_file_size = self.file.metadata().unwrap().len() as usize;
        let new_size = current_file_size + data.len();

        self.file.set_len(new_size as u64).unwrap();

        self.mmap = unsafe{MmapMut::map_mut(&self.file)?};

        self.mmap[current_file_size..new_size].copy_from_slice(data);

        Ok(())
    }
}