use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::io::{Read, Result};
use crate::constants::KILOBYTE;

const BUFFER_SIZE: usize = 4 * KILOBYTE;

#[derive(Debug)]
pub struct TcpProxy{
    port: u16,
    tcp_address: Ipv4Addr,
    sctp_proxy_address: SocketAddrV4,
}


impl TcpProxy{

    pub fn start(self) ->Result<()> {

        let browser_server = TcpListener::bind(SocketAddrV4::new(self.tcp_address, self.port))?;

        println!("Listening on {}:{}", self.tcp_address,self.port);

        for mut stream in browser_server.incoming(){

            let stream = stream?;

            Self::handle_client(stream)?

        }

        Ok(())
    }

    pub fn handle_client(mut stream: TcpStream) -> Result<()>{

        let mut buffer: Vec<u8> = vec![0;BUFFER_SIZE];

        let bytes_read = stream.read(&mut buffer)?;

        println!("Bytes read: {bytes_read:#?}");
        println!("Request: {}", String::from_utf8_lossy(&buffer));

        Ok(())
    }

}

pub struct TcpProxyBuilder{
    port: u16,
    tcp_address: Ipv4Addr,
    sctp_proxy_address: SocketAddrV4,
}

impl TcpProxyBuilder{
    pub fn new() -> Self{
        Self{
            port: 7878,
            tcp_address: Ipv4Addr::UNSPECIFIED,
            sctp_proxy_address: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED,7979),
        }
    }

    pub fn port(mut self, port: u16) -> Self{
        self.port = port;
        self
    }

    pub fn tcp_address(mut self, tcp_address: Ipv4Addr) -> Self{
        self.tcp_address = tcp_address;
        self
    }

    pub fn sctp_proxy_address(mut self, sctp_proxy_address: SocketAddrV4) -> Self{
        self.sctp_proxy_address = sctp_proxy_address;
        self
    }

    pub fn sctp_proxy_ipv4(mut self, sctp_proxy_ipv4: Ipv4Addr) -> Self{
        self.sctp_proxy_address.set_ip(sctp_proxy_ipv4);
        self
    }

    pub fn sctp_proxy_port(mut self,sctp_proxy_port: u16) -> Self{
        self.sctp_proxy_address.set_port(sctp_proxy_port);
        self
    }

    pub fn build(self) -> TcpProxy{

        TcpProxy{
            port: self.port,
            tcp_address: self.tcp_address,
            sctp_proxy_address: self.sctp_proxy_address,
        }
    }


}