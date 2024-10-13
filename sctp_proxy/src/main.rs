use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::{mem, thread};
use libc::{sa_family_t, AF_INET};
use utils::libc_wrappers::{debug_sockaddr, safe_inet_pton, SockAddrIn};
use utils::sctp_api::{safe_sctp_recvmsg, safe_sctp_sendmsg, safe_sctp_socket, SctpEventSubscribe, SctpPeer, SctpPeerBuilder};
use utils::sctp_client::{SctpClient, SctpClientBuilder};

const PORT: u16 = 7878;
const IPV4: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

fn main() {

    let mut events = SctpEventSubscribe::new();
    events.sctp_data_io_event = 1;

    let mut sctp_client = SctpClientBuilder::new()
        .socket()
        .address(IPV4)
        .port(PORT)
        .events(events)
        .build();

    sctp_client.connect();

    let tcp_server = TcpListener::bind(("0.0.0.0", PORT)).unwrap();

    for stream in tcp_server.incoming(){

        match stream {
            Ok(stream) => {
                handle_client(stream,&mut sctp_client);
            }
            Err(error) => {
                println!("Tcp stream error: {}", error);
            }
        }

    }
}

fn handle_client(mut stream: TcpStream, sctp_client: &mut SctpClient) {
    let mut buffer = [0;4096];

    loop{

        match stream.read(&mut buffer){
            Ok(0) =>{
                println!("Client disconnected");
                break;
            }

            Ok(n) => {
                let received_message = String::from_utf8_lossy(&buffer[..n]);
                println!("Client received message: {}", received_message);

                if let Err(error) = sctp_client.write(&mut buffer[..],n,&mut sctp_client.get_first_socket_address(),0,0,0){
                    println!("Client write error: {}", error);
                }

                let mut flags = 0;

                match sctp_client.read(&mut buffer[..],None,None,&mut flags){
                    Err(error)=>{
                        println!("Sctp read error: {}", error);
                    }

                    Ok(n) =>{
                        println!("Sctp received message: {}", String::from_utf8_lossy(&buffer[..n]));
                    }
                }

            }

            Err(e) =>{
                eprintln!("Client error: {}", e);
                break;
            }
        }

    }

}
