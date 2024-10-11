use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{mem, thread};
use libc::{sa_family_t, AF_INET};
use utils::libc_wrappers::{debug_sockaddr, safe_inet_pton, SockAddrIn};
use utils::sctp_api::{safe_sctp_sendmsg, safe_sctp_socket};

fn handle_client(mut client: TcpStream) {
    // Buffer pentru datele de la client
    let mut client_buf = [0; 4096];

    // Citim cererea de la client
    match client.read(&mut client_buf) {
        Ok(n) if n > 0 => {

            println!("Client read: {}", String::from_utf8_lossy(&client_buf[..]));

            let server_fd = safe_sctp_socket().unwrap();
            println!("SCTP SOCKET: {server_fd}");
            let mut server_addr: SockAddrIn = unsafe{mem::zeroed()};

            server_addr.sin_family = AF_INET as sa_family_t;
            server_addr.sin_port = 7878u16.to_be();

            let x = safe_inet_pton("127.0.0.1".to_string(),&mut server_addr.sin_addr.s_addr);
            println!("{x:?}");

            debug_sockaddr(&server_addr);

            let x = safe_sctp_sendmsg(server_fd,&client_buf,client_buf.len() as isize,&mut server_addr,0,0,0,0,0);

            println!("{x:?}");

            // // Ne conectăm la serverul țintă
            // if let Ok(mut server) = TcpStream::connect(server_addr) {
            //     // Trimitem cererea către server
            //     if server.write_all(&client_buf[..n]).is_ok() {
            //         // Buffer pentru răspunsul de la server
            //         let mut server_buf = [0; 4096];
            //
            //         // Citim răspunsul de la server
            //         if let Ok(m) = server.read(&mut server_buf) {
            //             if m > 0 {
            //                 // Trimitem răspunsul înapoi la client
            //                 let _ = client.write_all(&server_buf[..m]);
            //             }
            //         }
            //     }
            // }
        }
        Err(e) => eprintln!("Error reading from client: {:?}", e),
        _ => {}
    }
}

fn main() {
    // Ascultăm pe portul 8080 pentru conexiuni de la client
    let listener = TcpListener::bind("0.0.0.0:7879").expect("Could not bind to port 7879");


    println!("Proxy server listening on port 7879");

    // Acceptăm conexiuni de la clienți
    for stream in listener.incoming() {
        match stream {
            Ok(client) => {
                    handle_client(client);
            }
            Err(e) => eprintln!("Error accepting connection: {:?}", e),
        }
    }
}
