use std::borrow::Cow;
use scraper::{Html, Selector};
use std::fs;
use http::Request;
use utils::http_parsers::{basic_http_response, http_request_to_string, http_response_to_string, string_to_http_request, string_to_http_response};

#[derive(Debug)]
struct Obj{
    x:i32
}

fn main() {
    let str: Cow<str> = Cow::Owned("GET / HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nConnection: keep-alive\r\nCache-Control: max-age=0\r\nsec-ch-ua: \"Chromium\";v=\"130\", \"Google Chrome\";v=\"130\", \"Not?A_Brand\";v=\"99\"\r\nsec-ch-ua-mobile: ?0\r\nsec-ch-ua-platform: \"Linux\"\r\nUpgrade-Insecure-Requests: 1\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\nSec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-User: ?1\r\nSec-Fetch-Dest: document\r\nAccept-Encoding: gzip, deflate, br, zstd\r\nAccept-Language: en-US,en;q=0.9,ro-RO;q=0.8,ro;q=0.7,ko;q=0.6\r\n\r\n"
        .to_string());

    println!("{:?}",str);
    let response = string_to_http_request(str.as_ref());
    println!("response str: {:?}", response);
}

fn ceva(c:&str){
    let mut lines = c.lines();
    println!("{:?}",lines.next());
    println!("ceva is {}", c);
}

