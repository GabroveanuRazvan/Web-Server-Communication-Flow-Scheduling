use std::fs::File;
use std::io;
use std::os::fd::{FromRawFd, RawFd};
use libc::{O_CREAT, O_EXCL, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY};
use memmap2::{Mmap, MmapMut};
use crate::libc_wrappers::safe_shm_open;
use io::Result;

#[derive(Debug)]
pub struct MappedSharedMemory{
    fd: RawFd,
    name: String,
    size: usize,
    mmap: MmapMut,
}
#[derive(Debug)]
pub struct MappedSharedMemoryReadOnly{
    fd: RawFd,
    name: String,
    size: usize,
    mmap: Mmap
}

pub struct SharedMemory;

impl SharedMemory{
    /// Creates a new writeable mapped shared memory of fixed sized.
    /// Returns an error if the shared memory already exists.
    pub fn create(name: &str,mode: u32,size: usize) -> Result<MappedSharedMemory>{

        let fd = safe_shm_open(name,O_CREAT | O_EXCL | O_RDWR,mode)?;
        let file = unsafe { File::from_raw_fd(fd) };

        file.set_len(size as u64)?;

        let mmap = unsafe{MmapMut::map_mut(&file)?};

        Ok(MappedSharedMemory{
            fd,
            name: name.to_string(),
            size,
            mmap: mmap,
        })

    }

    /// Recreates a writeable mapped shared memory of fixed sized. Creates it if it does not exist.
    pub fn recreate(name: &str,mode: u32,size: usize) -> Result<MappedSharedMemory>{
        let fd = safe_shm_open(name,O_CREAT | O_TRUNC | O_RDWR,mode)?;

        let file = unsafe { File::from_raw_fd(fd) };

        file.set_len(size as u64)?;
        let mmap = unsafe{MmapMut::map_mut(&file)?};

        Ok(MappedSharedMemory{
            fd,
            name: name.to_string(),
            size,
            mmap: mmap,
        })
    }

    /// Opens an existing mapped shared memory in readonly mode.
    pub fn open(name:&str) -> Result<MappedSharedMemoryReadOnly>{
        let fd = safe_shm_open(name,O_RDONLY,0)?;
        let file = unsafe { File::from_raw_fd(fd) };
        let size = file.metadata()?.len() as usize;

        let mmap = unsafe{Mmap::map(&file)?};

        Ok(MappedSharedMemoryReadOnly{
            fd,
            name: name.to_string(),
            size,
            mmap: mmap,
        })
    }

    /// Opens an existing mapped shared memory in read-write mode.
    pub fn open_mut(name:&str) -> Result<MappedSharedMemory>{
        let fd = safe_shm_open(name,O_RDWR,0)?;
        let file = unsafe { File::from_raw_fd(fd) };
        let size = file.metadata()?.len() as usize;

        let mmap = unsafe{ MmapMut::map_mut(&file)? };

        Ok(MappedSharedMemory{
            fd,
            name: name.to_string(),
            size,
            mmap: mmap,
        })
    }
}