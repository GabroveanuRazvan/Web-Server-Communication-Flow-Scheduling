use std::str::FromStr;
use http::{Method, Request, Response, StatusCode, Uri};
use scraper::{Html, Selector};

pub fn parse_http_request(request_str: &str) -> Request<()> {
    let mut lines = request_str.lines();

    // First line contains the method, URI, and version
    let request_line = lines.next().unwrap();

    // get the parts of the first line
    let mut parts = request_line.split_whitespace();

    // extract the method
    let method = Method::from_str(parts.next().unwrap()).unwrap();

    // extract the uri
    let uri = Uri::from_str(parts.next().unwrap()).unwrap();

    // start constructing the request builder
    let mut request_builder = Request::builder()
        .method(method)
        .uri(uri);

    // extract the headers and add them to the builder
    for line in lines {

        // if the line is empty it means we have reached the end of the headers
        if line.is_empty() {
            break;
        }

        // split after ": " and extract the name and value
        let mut header_parts = line.splitn(2, ": ");
        let name = header_parts.next().unwrap();
        let value = header_parts.next().unwrap();

        // add the header
        request_builder = request_builder.header(name, value);
    }

    // consume the builder and return the request
    request_builder.body(()).unwrap()
}

pub fn response_to_string(response: &Response<()>) -> String {

    // construct the status line and the reason if known
    let status_line = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status().as_u16(),
        response.status().canonical_reason().unwrap_or("Unknown canonical reason")
    );

    // construct the headers
    let mut headers = String::new();
    for (key, value) in response.headers() {
        headers.push_str(&format!(
            "{}: {}\r\n",
            key.as_str(),
            value.to_str().unwrap_or("")
        ));
    }

    // add the carriages to mark the end of the headers
    headers.push_str("\r\n");

    // concatenate the status line and headers
    status_line + &headers
}

pub fn basic_http_response(content_length: usize) -> Response<()>{
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .header("Connection", "keep-alive")
        .header("Keep-Alive", "timeout=5,max=1")
        .header("Content-Length", content_length.to_string())
        .body(())
        .unwrap()
}

pub fn extracts_http_paths(html_content: String) -> Vec<String> {

    let document = Html::parse_document(&html_content);

    let href_selector = Selector::parse("[href]").unwrap();
    let src_selector = Selector::parse("[src]").unwrap();

    let mut paths =  Vec::new();

    for element in document.select(&href_selector) {
        if let Some(href) = element.value().attr("href") {
            paths.push(href.to_string());
        }
    }

    for element in document.select(&src_selector) {
        if let Some(src) = element.value().attr("src") {
            paths.push(src.to_string());
        }
    }

    paths
}