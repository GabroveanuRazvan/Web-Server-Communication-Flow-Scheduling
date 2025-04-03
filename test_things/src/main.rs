use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use path_clean::PathClean;
use utils::logger::Logger;

fn main(){

    // let a = PathBuf::from("/a/b");
    // let b = PathBuf::from("../d/e");
    //
    // let c = a.parent().unwrap().join(b);
    // println!("{:?}",c);
    // println!("{:?}",c.clean());


    let a = PathBuf::from("/a");
    let b = PathBuf::from("/b");
    println!("{:?}",a.join(b))

}

