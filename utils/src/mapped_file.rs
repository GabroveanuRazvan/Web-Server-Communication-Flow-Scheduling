use std::cmp::Ordering;
use std::fs::File;
use std::io;
use memmap2::MmapMut;

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

impl PartialEq for MappedFile{
    fn eq(&self, other: &Self) -> bool{
        self.file_size == other.file_size
    }
}

impl Eq for MappedFile{}

impl PartialOrd for MappedFile{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>{
        Some(self.cmp(other))
    }
}

impl Ord for MappedFile{
    fn cmp(&self, other: &Self) -> Ordering{
        self.file_size.cmp(&other.file_size)
    }
}