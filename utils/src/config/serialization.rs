use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter, Result};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

/// Saves a serializable object into the given file path.
pub fn save<T,P>(object: T, file_path: P) -> Result<()>
    where T: Serialize, P: AsRef<Path>
{

    match file_path.as_ref().parent(){
        None => (),
        Some(parent) => create_dir_all(parent)?,
    }

    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &object)?;

    Ok(())
}

/// Loads an object from a json file.
pub fn load<T,P>(file_path: P) -> Result<T>
    where T: Serialize + DeserializeOwned, P: AsRef<Path>
{

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}

#[cfg(test)]

mod tests{
    use std::collections::HashSet;
    use std::fs;
    use std::net::Ipv4Addr;
    use super::*;

    #[test]
    fn test_save_1(){

        let mut set = HashSet::<i32>::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        fs::create_dir(PathBuf::from("./tests")).unwrap_or_default();
        save(set,PathBuf::from("./tests/test_save1.json")).unwrap();

        let assert = fs::exists(PathBuf::from("./tests/test_save1.json")).unwrap_or_else(|_|false);

        assert!(assert);

    }

    #[test]
    fn test_save_2(){

        let mut set = HashSet::<Ipv4Addr>::new();

        set.insert(Ipv4Addr::new(127, 0, 0, 1));
        set.insert(Ipv4Addr::new(192, 168, 0, 2));
        set.insert(Ipv4Addr::new(99, 124, 22, 3));

        fs::create_dir(PathBuf::from("./tests")).unwrap_or_default();
        save(set,PathBuf::from("./tests/test_save2.json")).unwrap();

        let assert = fs::exists(PathBuf::from("./tests/test_save2.json")).unwrap_or_else(|_|false);

        assert!(assert);

    }

    #[test]
    fn test_load_1(){

        let mut set = load(PathBuf::from("./tests/test_load1.json")).unwrap_or_else(|_| HashSet::<i32>::new());

        assert!(set.contains(&2));
        assert!(set.contains(&4));
        assert!(set.contains(&6));
    }

    #[test]
    fn test_load_2(){
        let mut set = load(PathBuf::from("./tests/test_load2.json")).unwrap_or_else(|_| HashSet::<Ipv4Addr>::new());

        assert!(set.contains(&Ipv4Addr::new(127, 0, 0, 1)));
        assert!(set.contains(&Ipv4Addr::new(127, 0, 0, 2)));
        assert!(set.contains(&Ipv4Addr::new(127, 0, 0, 3)));

    }
}