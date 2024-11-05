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

    let s= "1".to_string();

}


