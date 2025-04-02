use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::Mutex;
use std::io::Result;
use std::io::Write;

/// Simple logger for concurrent access to a single file.
pub struct Logger{
    file: Mutex<File>,
}

impl Logger{

    /// Creates the file if it does not exist, otherwise truncates it.
    pub fn new<T>(file_path: T) -> Result<Self>
        where T: AsRef<Path> {

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