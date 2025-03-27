use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

fn main(){

    let child = Command::new("../tcp_cache_fetcher_proxy/target/debug/tcp_cache_fetcher_proxy")
        .stdout(Stdio::piped())
        .spawn().unwrap();

    let reader = BufReader::new(child.stdout.unwrap());

    for line in reader.lines() {
        println!("{}", line.unwrap());
    }

}

