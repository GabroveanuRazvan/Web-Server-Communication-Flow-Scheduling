use std::collections::HashMap;
use std::str::FromStr;
use http::{Method, Request, Response, StatusCode, Uri};
use scraper::{Html, Selector};

pub fn string_to_http_request(request_str: &str) -> Request<()> {
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

pub fn http_request_to_string(request: Request<()>) -> String {

    // build the status line: METHOD URI VERSION\r\n
    let status_line = format!(
        "{} {} HTTP/1.1\r\n",
        request.method(),
        request.uri()
    );

    // build the headers: header_name: header_value\r\n
    let mut headers = String::new();
    for (key, value) in request.headers() {
        headers.push_str(&format!(
            "{}: {}\r\n",
            key.as_str(),
            value.to_str().unwrap_or("")
        ));
    }

    // the headers end with \r\n
    headers.push_str("\r\n");

    // add the carriages to mark the end of the headers
    status_line + &headers
}

pub fn http_response_to_string(response: Response<()>) -> String {

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

pub fn string_to_http_response(response_str: &str) -> Response<()> {

    let mut lines = response_str.lines();

    // get the status line
    let status_line = lines.next().unwrap();
    // split the status line
    let mut status_parts = status_line.split_whitespace();

    // the status line consists of VERSION STATUS_CODE STATUS, we just need the status code
    let _version = status_parts.next();
    let status_code_str = status_parts.next().unwrap();

    // Parse the code into an integer
    let status_code = status_code_str.parse::<u16>().unwrap();
    let status = StatusCode::from_u16(status_code).unwrap();

    // build the headers: header_name: header_value\r\n
    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            // reached the end of headers
            break;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() == 2 {
            headers.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    // Build the response
    let mut response_builder = Response::builder().status(status);

    for (key, value) in headers {
        response_builder = response_builder.header(&key, value);
    }

    response_builder.body(()).unwrap()
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

pub fn basic_http_get_request(uri: &str) -> Request<()> {

    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(()).unwrap()

}

/// Parses the html file and extracts all afferent paths from href and src selectors.
pub fn extract_http_paths(html_content: &str) -> Vec<String> {

    // Parse the html document
    let document = Html::parse_document(&html_content);

    // Prepare the selectors
    let selectors: [(Selector,&str);2] = [
        (Selector::parse("[href]").unwrap(),"href"),
        (Selector::parse("[src]").unwrap(),"src"),
    ];

    let mut paths =  Vec::new();

    for (selector,attribute) in selectors{

        for element in document.select(&selector) {

            if let Some(href) = element.value().attr(attribute) {

                let mut path = href.to_string();

                // Skip link paths
                if path.starts_with("https"){
                    continue;
                }

                // Add / to path for consistency
                if !path.starts_with('/'){
                    path = format!("/{}",path);
                }

                paths.push(path);
            }

        }

    }
    paths
}

/// Encodes a path by replacing "/" with "__".
pub fn encode_path(path: &str) -> String {
    path.replace("/", "__")
}

/// Decodes a path by replacing "__" with "/".
pub fn decode_path(encoded: &str) -> String {
    encoded.replace("__", "/")
}

/// Extracts the uri of a request line from an HTTP request.
pub fn extract_uri(line: String) -> Option<String>{
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() > 2{

        let uri = parts[1];

        match uri.strip_prefix("/"){
            Some("") => Some("/index.html".to_string()),
            Some(_) => Some(uri.to_string()),
            None => None
        }

    }else{
        None
    }
}