use inotify::{EventMask, Inotify, WatchMask};
use std::io::{Read, Write};
use std::net::TcpStream;

fn encode_path(path: &str) -> String {
    path.replace("/", "__")
}

fn decode_path(encoded: &str) -> String {
    encoded.replace("__", "/")
}

fn main(){

    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();

    stream.write_all("/images_4k/4k1.jpg".as_bytes()).unwrap();

}
