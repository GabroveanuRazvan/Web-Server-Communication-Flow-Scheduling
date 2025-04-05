use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use path_clean::PathClean;
use utils::logger::Logger;

fn main(){

    let mut x = 255u8;

    x = x.wrapping_add(2);
    println!("{}",x);

}

