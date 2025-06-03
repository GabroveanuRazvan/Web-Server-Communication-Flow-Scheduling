use std::collections::{HashMap, HashSet};
use std::{fs, io};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use io::Result;
use memmap2::Mmap;
use path_clean::PathClean;
use crate::http_parsers::extract_http_paths;


/// Service used to build a map of html files as keys, mapped to a vector of files that are referenced in the key html file.
pub struct HtmlPrefetchService<T: AsRef<Path> + Clone> {

    root: T,
    html_links: HashMap<String,Vec<String>>,

}

impl<T: AsRef<Path> + Clone> HtmlPrefetchService<T> {

    /// Creates a new service.
    pub fn new(root: T) -> Self{
        Self{
            root,
            html_links: HashMap::new()
        }
    }
    
    fn clean_links(&mut self){
         
        let new_links: HashMap<String,Vec<String>> = self.html_links.iter().map(|(key, paths)| {
            
            let key = PathBuf::from(key);
            let key = key.strip_prefix(&self.root).unwrap();
            let new_key = String::from("/") + &key.display().to_string();
            
            let paths: Vec<String> = paths.into_iter().map(|path|{
                let path = PathBuf::from(path);
                let stripped_path = path.strip_prefix(&self.root).unwrap();

                let html_path = String::from("/");
                html_path + &stripped_path.display().to_string()
            }).collect();


            (new_key,paths)
            
        }).collect();
        
        self.html_links = new_links;
        
    }
    
    
    pub fn build_prefetch_links(&mut self)-> Result<()>{
        
        self.build_prefetch_links_helper(self.root.clone())?;
        self.clean_links();
        
        Ok(())
        
    }
    
    fn build_prefetch_links_helper(&mut self,root: impl AsRef<Path>) -> Result<()> {
    
        // Read the root directory and iterate over its files
        let entry_it = fs::read_dir(&root)?;
        
        for entry in entry_it {
            let entry_path = entry?.path();
        
            // Recursively call the function to traverse the whole directory tree when there are more directories
            if entry_path.is_dir(){
                self.build_prefetch_links_helper(&entry_path)?;
                continue;
            }
        
            // Check the file extension
            if let Some(extension) = entry_path.extension()  {
        
                if extension != "html"{
                    continue;
                }
        
                // Read the html file
                let file = OpenOptions::new()
                    .read(true)
                    .write(false)
                    .create(false)
                    .open(&entry_path)?;
        
                let file_parent = entry_path.parent().unwrap();
        
                let mmap = unsafe{Mmap::map(&file)?};
                let file_content = std::str::from_utf8(&mmap).unwrap();
        
                // Extract the file dependencies if they exist
                let dependencies = extract_http_paths(file_content);
        
                if dependencies.is_empty(){
                    continue;
                }
        
                //Get the unique file paths
                let unique_dependencies: HashSet<PathBuf> = dependencies.into_iter()
                    .map(|path| PathBuf::from(path))
                    .collect();
        
                let entry_parent = entry_path.parent().unwrap();
                let dependencies: Vec<String> = unique_dependencies.into_iter().
                    map(|path|{
                        let whole_path = PathBuf::from(entry_parent).join(path);
                        let path = PathBuf::from(&whole_path).clean();
                        path.display().to_string()
                    }).collect();
        
                
                // Insert the new entry
                self.html_links.insert(entry_path.clean().display().to_string(), dependencies);
        
                
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
    pub fn get_links(self) -> HashMap<String, Vec<String>>{
        self.html_links
    }

}