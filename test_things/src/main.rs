use std::env;
use memmap2::Mmap;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {

    let current_dir = env::current_dir()?;
    println!("Current directory: {}", current_dir.display());



    // Deschidem fișierul
    let file = File::open("./web_files/ceva.html")?;

    // Mapăm fișierul în memorie
    let mmap = unsafe { Mmap::map(&file)? };

    println!("Mmap content: {:?}", mmap);

    // Citim conținutul fișierului din memoria mapată
    let file_content = std::str::from_utf8(&mmap).expect("Fisierul nu este un UTF-8 valid");
    println!("{}", file_content);

    let chunk_size = 5;

    for chunk in mmap.chunks(chunk_size) {
        println!("Chunk size: {}", chunk.len());
        println!("Chunk content: {:?}", String::from_utf8_lossy(&chunk));
    }


    Ok(())
}
