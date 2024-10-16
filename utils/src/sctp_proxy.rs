use std::{io, mem};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use crate::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder};
use crate::sctp_client::SctpStreamBuilder;
use io::Result;
use std::io::Read;
use crate::libc_wrappers::{debug_sctp_sndrcvinfo, new_sctp_sndrinfo, SctpSenderInfo};

const BUFFER_SIZE: usize = 2048;

/// Abstraction for a tcp to sctp proxy
/// The tcp server will listen on a given address and redirect its data to the sctp client
/// The client will connect to the sctp-server using its addresses and send the data to be processes
pub struct SctpProxy{

    port: u16,
    sctp_address: Ipv4Addr,
    sctp_peer_addresses: Vec<Ipv4Addr>,
    tcp_address: Ipv4Addr,
}

impl SctpProxy{
    /// Method that starts the proxy
    pub fn start(self) -> Result<()>{

        let mut tcp_server =TcpListener::bind((self.tcp_address.to_string(),self.port))?;

        println!("Sctp Proxy started.");

        for stream in tcp_server.incoming(){

            let stream = stream?;

            self.handle_client(stream);

        }

        Ok(())
    }

    /// Client handler method
    fn handle_client(&self, mut stream: TcpStream) {

        // create a new sctp client

        let events = SctpEventSubscribeBuilder::new().sctp_data_io_event().build();

        let mut sctp_client = SctpStreamBuilder::new()
            .socket()
            .port(self.port)
            .address(self.sctp_address)
            .addresses(self.sctp_peer_addresses.clone())
            .ttl(0)
            .events(events)
            .build();

        sctp_client.connect();
        sctp_client.options();

        println!("New client");
        println!("{sctp_client:?}");

        let mut buffer: Vec<u8> = vec![0;BUFFER_SIZE];

        loop{

            match stream.read(&mut buffer){

                Ok(0) => {
                    println!("Tcp client closed");
                    break;
                }

                Err(error) => {
                    eprintln!("Tcp Client error: {}", error);
                    break;
                }

                Ok(n) => {
                    let received_message = String::from_utf8_lossy(&buffer[..n]);

                    println!("Got Bytes: {n}");
                    println!("Tcp Client received message: {}", received_message);

                    if let Err(error) = sctp_client.write(&mut buffer[..],n,0,0){
                        panic!("Sctp Client write error: {}", error);
                    }

                    let mut sender_info = new_sctp_sndrinfo();

                    match sctp_client.read(&mut buffer,Some(&mut sender_info),None){
                        Err(error)=>{
                            panic!("Sctp read error: {}", error);
                        }

                        Ok(n) =>{
                            debug_sctp_sndrcvinfo(&sender_info);
                            println!("Sctp received message: {}", String::from_utf8_lossy(&buffer[..n]));
                        }
                    }
                }

            }

        }

    }
}


/// Builder pattern for SctpProxy

pub struct SctpProxyBuilder{

    port: u16,
    sctp_address: Ipv4Addr,
    sctp_peer_addresses: Vec<Ipv4Addr>,
    tcp_address: Ipv4Addr,
}

impl SctpProxyBuilder {

    /// Creates a new builder for the proxy
    pub fn new() -> Self {

        Self{
            port: 0,
            sctp_address: Ipv4Addr::new(0, 0, 0, 0),
            sctp_peer_addresses: vec![],
            tcp_address: Ipv4Addr::new(0, 0, 0, 0),
        }
    }

    /// Sets the port
    pub fn port(mut self, port: u16) -> Self {

        self.port = port;
        self
    }

    /// Sets the addresses of the sctp client
    pub fn sctp_peer_addresses(mut self, addresses: Vec<Ipv4Addr>) -> Self {

        self.sctp_peer_addresses = addresses;
        self
    }

    /// Sets the address that will be used to send data
    pub fn sctp_address(mut self, address: Ipv4Addr) -> Self {

        self.sctp_address = address;
        self
    }

    /// Sets the address that the tcp server will listen to
    pub fn tcp_address(mut self, address: Ipv4Addr) -> Self {

        self.tcp_address = address;
        self
    }

    /// Builds the proxy based on the given data
    pub fn build(self) -> SctpProxy{

        SctpProxy{
            port: self.port,
            sctp_address: self.sctp_address,
            sctp_peer_addresses: self.sctp_peer_addresses,
            tcp_address: self.tcp_address,
        }
    }
}