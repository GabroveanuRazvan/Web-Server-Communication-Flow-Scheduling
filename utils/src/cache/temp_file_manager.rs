use std::collections::HashMap;
use std::fs::{create_dir, remove_dir_all, remove_file, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use chrono::Utc;

/// Manager for creating, evicting, opening temporary files
#[derive(Debug)]
pub struct TempFileManager{

    dir_path: PathBuf,
    mapped_files: HashMap<String,String>,
}

impl TempFileManager {

    /// Creates a new directory from the given path
    pub fn new(dir_path: &Path) -> Self {

        if let Err(error) = create_dir(dir_path){
            panic!("Error creating temp directory: {}",error)
        }

        Self{
            dir_path: PathBuf::from(dir_path),
            mapped_files: HashMap::new(),
        }

    }

    /// Creates a temporary file into the given directory using the unique id
    pub fn add(&mut self,key: String) -> io::Result<()>{

        let unique_name = Self::unique_name();
        let file_path = self.dir_path.join(&unique_name);

        File::create(file_path)?;

        self.mapped_files.insert(key,unique_name);

        Ok(())
    }

    /// Evicts a chosen file based on its id if the file exists
    pub fn evict(&mut self,key: &String) -> io::Result<()>{

        let file_name = match self.mapped_files.get(key){
            Some(value) => Ok(value),
            None => Err(io::Error::new(io::ErrorKind::NotFound,"Key not found"))
        }?;

        let file_path = self.dir_path.join(file_name);

        self.mapped_files.remove(key);

        remove_file(file_path)
    }


    /// Opens a file for reading and writing; if the file does not exist it is created using the given id
    pub fn open(&mut self, key: String) -> io::Result<File>{

        if !self.mapped_files.contains_key(&key) {
            self.add(key.clone())?;
        }

        let file_name = self.mapped_files.get(&key).unwrap();
        let file_path = self.dir_path.join(file_name);

        OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .truncate(false)
            .open(file_path)

    }

    /// Checks if the temporary file with the given id exists
    pub fn contains(&mut self,key: String) -> bool {
        self.mapped_files.contains_key(&key)
    }

    /// Returns a unique string representing the number of non-leap-nanoseconds since January 1, 1970 UTC used for naming the temporary files
    pub fn unique_name() -> String {
        Utc::now().timestamp_nanos_opt().unwrap().to_string()
    }

}

/// When the manager goes out of scope, the directory and all of its contents are deleted
impl Drop for TempFileManager {
    fn drop(&mut self) {
        remove_dir_all(&self.dir_path).unwrap();
    }
}
