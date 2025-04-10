use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::thread;
use path_clean::PathClean;
use utils::logger::Logger;


fn child_job(s: &mut String) {
    *s = s.to_uppercase();
}
fn main(){

    let arr = [12;5];

}

