use std::str::FromStr;
use http::{HeaderMap, Method, Request, Response, StatusCode, Uri};

fn main() {
    // Un exemplu de cerere HTTP sub formă de string
    let http_request = "GET /index.html HTTP/1.1\r\nHost: localhost\r\nUser-Agent: Rust\r\n\r\n";

    // Parsem string-ul în Request
    let request = parse_http_request(http_request);

    println!("{request:?}");

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .header("Connection", "keep-alive")
        .header("Content-Length", "0")
        .body(())
        .unwrap();

    // Convertește răspunsul în string
    let response_string = response_to_string(&response);

    println!("{response_string}");
}

fn parse_http_request(request_str: &str) -> Request<()> {
    let mut lines = request_str.lines();

    // Prima linie conține metoda, URI-ul și versiunea HTTP
    let request_line = lines.next().unwrap();
    let mut parts = request_line.split_whitespace();

    // Extragem metoda (GET, POST, etc.)
    let method = Method::from_str(parts.next().unwrap()).unwrap();

    // Extragem URI-ul
    let uri = Uri::from_str(parts.next().unwrap()).unwrap();

    // Construim obiectul Request cu headere
    let mut request_builder = Request::builder()
        .method(method)
        .uri(uri);

    // Extragem și adăugăm headerele
    for line in lines {
        if line.is_empty() {
            break; // Linie goală marchează sfârșitul headerelor
        }

        let mut header_parts = line.splitn(2, ": ");
        let name = header_parts.next().unwrap();
        let value = header_parts.next().unwrap();

        // Adaugă fiecare header folosind apeluri separate la header()
        request_builder = request_builder.header(name, value);
    }

    // Finalizează construirea cererii
    request_builder.body(()).unwrap()
}

fn response_to_string(response: &Response<()>) -> String {
    // Construiește linia de status
    let status_line = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status().as_u16(),
        response.status().canonical_reason().unwrap_or("")
    );

    // Construiește headerele
    let mut headers = String::new();
    for (key, value) in response.headers() {
        headers.push_str(&format!(
            "{}: {}\r\n",
            key.as_str(),
            value.to_str().unwrap_or("")
        ));
    }

    // Adaugă linia goală care separă headerele de body
    headers.push_str("\r\n");

    // Concatenăm linia de status și headerele, fără body
    status_line + &headers
}