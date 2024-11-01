use std::{io, mem,thread};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use crate::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder, MAX_STREAM_NUMBER};
use crate::sctp_client::{SctpStream, SctpStreamBuilder};
use io::Result;
use std::io::{Read, Write};
use crate::libc_wrappers::{debug_sctp_sndrcvinfo, new_sctp_sndrinfo, SctpSenderInfo};

const BUFFER_SIZE: usize = 4096;

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

        println!("Sctp Proxy started and listening on {:?}:{}",self.tcp_address,self.port);
        println!("Messages redirected to: {:?}:{}",self.sctp_address,self.port);
        println!("Connect by: http://127.0.0.1:{}",self.port);

        for stream in tcp_server.incoming(){

            let stream = stream?;

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

            //TODO thread pool

                Self::handle_client(stream,sctp_client)


        }

        Ok(())
    }

    /// Client handler method
    fn handle_client(mut tcp_stream: TcpStream, mut sctp_client: SctpStream) {

        // used to RR over the streams
        let mut stream_number = 0u16;

        println!("New client");

        let mut buffer: Vec<u8> = vec![0;BUFFER_SIZE];

        loop{

            println!("Tcp listener waiting for messages...");
            // the tcp stream waits for a request
            match tcp_stream.read(&mut buffer){

                Ok(0) => {
                    println!("Tcp client closed");
                    break;
                }

                Err(error) => {
                    panic!("Tcp Client error: {}", error);
                }

                // request received
                Ok(n) => {
                    let received_message = String::from_utf8_lossy(&buffer[..n]);

                    println!("Got Bytes: {n}");
                    println!("Tcp Client received message:\n{}", received_message);

                    // send the request to server via sctp-stream
                    if let Err(error) = sctp_client.write(&mut buffer[..],n,stream_number,0){
                        panic!("Sctp Client write error: {}", error);
                    }

                    // simple RR over the streams
                    stream_number = (stream_number + 1) % MAX_STREAM_NUMBER;

                    let mut sender_info = new_sctp_sndrinfo();

                    loop{
                        // the sctp-stream waits to get a response
                        match sctp_client.read(&mut buffer,Some(&mut sender_info),None){
                            // end message received
                            Ok(1) => {
                                println!("Sctp client ended processing");
                                break;
                            }

                            Err(error)=>{
                                panic!("Sctp read error: {}", error);
                            }

                            // response received
                            Ok(n) =>{

                                debug_sctp_sndrcvinfo(&sender_info);
                                // write into tcp stream
                                tcp_stream.write(&buffer[..n]);
                                println!("Sctp received message of size {n}:\n{}", String::from_utf8_lossy(&buffer[..n]));
                            }
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