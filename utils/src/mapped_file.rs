use std::cmp::Ordering;
use std::fs::File;
use std::io;
use memmap2::MmapMut;
use crate::http_parsers::{basic_http_response, http_response_to_string};
use crate::sctp_client::SctpStream;
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

const CHUNK_SIZE: usize = 2048;

pub struct MappedFileJob<'a> {
    mapped_file: MappedFile,
    stream: &'a SctpStream
}

impl<'a> MappedFileJob<'a>{
    pub fn new(mapped_file: MappedFile,sctp_stream: &'a SctpStream) -> Self{
        Self{
            mapped_file,
            stream: sctp_stream
        }
    }

    pub fn mmap_as_slice(&self) -> &[u8]{
        &self.mapped_file.mmap
    }
}

impl<'a>  Job for MappedFileJob<'a>{
    fn execute(mut self) {
        let response_body_size = self.mapped_file.file_size();

        let mut response_bytes = http_response_to_string(basic_http_response(response_body_size)).into_bytes();
        let response_size = response_bytes.len();

        // send the header of the html response
        match self.stream.write(&mut response_bytes,response_size,0,2){
            Ok(bytes) => println!("Wrote {bytes}"),
            Err(e) => println!("Write Error: {:?}",e)
        }

        // send the body of the response
        match self.stream.write_chunked(&self.mapped_file.mmap_as_slice(),CHUNK_SIZE,0,2){
            Ok(bytes) => println!("Wrote {bytes}"),
            Err(e) => println!("Write Error: {:?}",e)
        }

        // send a null character to mark the end of the message
        match self.stream.write_null(0,2){
            Ok(bytes) => println!("Wrote {bytes}"),
            Err(e) => println!("Write Error: {:?}",e)
        }
    }
}

impl<'a>  PartialEq for MappedFileJob<'a> {
    fn eq(&self, other: &Self) -> bool{
        self.mapped_file.file_size == other.mapped_file.file_size
    }
}

impl <'a>Eq for MappedFileJob<'a>{}

impl <'a> PartialOrd for MappedFileJob<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>{
        Some(self.cmp(other))
    }
}

impl <'a> Ord for MappedFileJob<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering{
        self.mapped_file.file_size.cmp(&other.mapped_file.file_size)
    }
}
















