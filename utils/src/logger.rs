use std::fs;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::Mutex;
use std::io::{ErrorKind, Result};
use std::io::Write;

/// Simple logger for concurrent access to a single file.
pub struct Logger{
    file: Mutex<File>,
}

impl Logger{

    /// Creates the file if it does not exist, otherwise truncates it.
    pub fn new<T>(file_path: T) -> Result<Self>
        where T: AsRef<Path> {
        
        let path_parent = file_path.as_ref().parent().expect("Invalid file path");
        
        match fs::create_dir_all(path_parent){
            Err(error) if error.kind() != ErrorKind::AlreadyExists => return Err(error),
            _ => ()
        }
        
        Ok(Self{
            file: Mutex::new(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(file_path)?
            )
        })

    }

    /// Writes a new line into the logging file.
    pub fn writeln(&self, message: &str) -> Result<()>{

        let mut file_guard = self.file.lock().unwrap();
        writeln!(file_guard,"{}",message)

    }

}