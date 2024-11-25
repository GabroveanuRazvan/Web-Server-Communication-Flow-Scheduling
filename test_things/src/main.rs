use std::io;
use std::thread::sleep;
use utils::cache::lru_cache::TempFileCache;
use utils::constants::{BYTE, KILOBYTE};

fn main() ->Result<(),std::io::Error>{

    let mut cache = TempFileCache::new(10 * BYTE);

    cache.insert("mere".to_string());
    cache.write_append("mere",b"12345")?;

    cache.insert("mere2".to_string());
    cache.write_append("mere2",b"12345")?;

    cache.insert("altceva".to_string());
    cache.write_append("altceva",b"123456789")?;

    println!("{:?}",cache);

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input) // Citim input-ul de la tastatură
        .expect("Eroare la citirea liniei");


    Ok(())
}
