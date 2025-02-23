use std::collections::{HashMap, HashSet};
use std::{fs, io};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use io::Result;
use memmap2::Mmap;
use path_clean::PathClean;
use crate::http_parsers::extract_http_paths;


/// Service used to build a map of html files as keys, mapped to a vector of files that are referenced in the key html file.
pub struct HtmlPrefetchService {

    html_links: HashMap<PathBuf,Vec<PathBuf>>

}

impl HtmlPrefetchService {

    /// Creates a new service.
    pub fn new() -> Self{
        Self{
            html_links: HashMap::new()
        }
    }

    /// Recursively walk across the root directory and process each file.
    /// Each html file will be parsed, and each used file path will be stored as an entry into the html_links map.
    pub fn build_prefetch_links<T: AsRef<Path>>(&mut self, root: T) -> Result<()>{

        // Read the root directory and iterate over its files
        let entry_it = fs::read_dir(root)?;

        for entry in entry_it {
            let path = entry?.path();

            // Recursively call the function to traverse the whole directory tree when there are more directories
            if path.is_dir(){
                self.build_prefetch_links(&path)?;
                continue;
            }

            // Check the file extension
            if let Some(extension) = path.extension()  {

                if extension == "html"{

                    // Read the html file
                    let file = OpenOptions::new()
                        .read(true)
                        .write(false)
                        .create(false)
                        .open(&path)?;

                    let file_parent = path.parent().unwrap();

                    let mmap = unsafe{Mmap::map(&file)?};
                    let file_content = std::str::from_utf8(&mmap).unwrap();

                    // Extract the file dependencies if they exist
                    let dependencies = extract_http_paths(file_content);

                    if dependencies.is_empty(){
                        continue;
                    }

                    //Get the unique file paths
                    let unique_dependencies = dependencies.iter()
                        .map(|path| file_parent.join(path))
                        .collect::<HashSet<PathBuf>>();

                    // Get a vector of the file sizes
                    let dependencies_sizes: Vec<u64> = unique_dependencies.iter()
                        .filter_map(Self::get_file_size)
                        .collect();

                    // Pair the file paths with their sizes
                    let mut paired_dependencies: Vec<_> = unique_dependencies.iter().zip(dependencies_sizes.iter()).collect();
                    paired_dependencies.sort_by_key(|&(_path,size)| size);

                    // Sort the file paths based on their sizes; also clean the paths
                    let sorted_dependencies: Vec<_> = paired_dependencies.iter()
                        .map(|&(path,_size)| path.clone().clean() )
                        .collect();

                    // Insert the new entry
                    self.html_links.insert(path.clean(),sorted_dependencies);


                }

            }

        }

        Ok(())
    }

    /// Closure used in a filter_map() call. Returns the file size of a path if the file size can be read.
    fn get_file_size(path : impl AsRef<Path>) -> Option<u64>{
        let file = match File::open(path){
            Ok(f) => f,
            Err(_) => return None,
        };

        let file_size = match file.metadata(){

            Ok(metadata) => metadata.len(),
            Err(_) => return None

        };

        Some(file_size)
    }

    /// Consumes the service, returning the built map
    pub fn get_links(self) -> HashMap<PathBuf, Vec<PathBuf>>{
        self.html_links
    }

}