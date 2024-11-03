use scraper::{Html, Selector};
use std::fs;
use utils::http_parsers::extracts_http_paths;

fn main() {
    // Citește conținutul fișierului HTML
    let html_content = fs::read_to_string("web_files/index.html").expect("Nu am putut citi fișierul HTML");



    println!("{:?}",extracts_http_paths(html_content));
}
