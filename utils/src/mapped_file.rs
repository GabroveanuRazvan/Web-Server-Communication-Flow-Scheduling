use std::fs::File;
use std::io;
use memmap2::MmapMut;
use crate::shortest_job_first_pool::Job;

/// Data structure used to store a file and its mapped content
#[derive(Debug)]
pub struct MappedFile{
    file: File,
    file_size: usize,
    mmap: MmapMut,
}

impl MappedFile{
    /// Create a new map from a given file
    pub fn new(file: File) -> io::Result<Self>{

        let mmap = unsafe{MmapMut::map_mut(&file)?};
        let file_size = file.metadata()?.len() as usize;

        Ok(Self{
            file,
            file_size,
            mmap,
        })

    }

    /// Used to flush the written data on the disk
    pub fn flush(&mut self) -> io::Result<()>{
        self.mmap.flush()
    }

    /// Writes the new data at the end of the file
    pub fn write_append(&mut self,data: &[u8]) -> io::Result<()>{

        let old_size = self.file_size;

        self.file_size += data.len();

        self.file.set_len(self.file_size as u64).unwrap();

        self.mmap = unsafe{MmapMut::map_mut(&self.file)?};

        self.mmap[old_size..self.file_size].copy_from_slice(data);

        Ok(())
    }

    /// Getter for the raw slice of bytes of the file
    pub fn mmap_as_slice(&self) -> &[u8]{
        &self.mmap
    }

    /// Getter for the file size
    pub fn file_size(&self) -> usize{
        self.file_size
    }

}

pub struct MappedFileJob{
    mapped_file: MappedFile,
    job: Box<dyn FnOnce() + Send + 'static>,
}

impl MappedFileJob{
    pub fn new(mapped_file: MappedFile,job: Box<dyn FnOnce() + Send + 'static>) -> Self{
        Self{
            mapped_file,
            job,
        }
    }

    pub fn mmap_as_slice(&self) -> &[u8]{
        &self.mapped_file.mmap
    }
}

impl Job for MappedFileJob{
    fn execute(mut self) {
        (self.job)();
    }
}

impl PartialEq for MappedFileJob{
    fn eq(&self, other: &Self) -> bool{
        self.mapped_file.file_size == other.mapped_file.file_size
    }
}

impl Eq for MappedFileJob{}

impl PartialOrd for MappedFileJob{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>{
        Some(self.cmp(other))
    }
}

impl Ord for MappedFileJob{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering{
        self.mapped_file.file_size.cmp(&other.mapped_file.file_size)
    }
}
















