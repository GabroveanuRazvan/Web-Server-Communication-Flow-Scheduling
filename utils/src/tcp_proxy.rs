use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Read, Result};
use std::path::Path;
use crate::constants::KILOBYTE;
use crate::http_parsers::encode_path;

const BUFFER_SIZE: usize = 4 * KILOBYTE;

const CACHE_PATH: &str = "/tmp/tmpfs";
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
            let proxy_stream = TcpStream::connect(self.sctp_proxy_address)?;

            Self::handle_client(stream,proxy_stream)?

        }

        Ok(())
    }

    pub fn handle_client(stream: TcpStream,proxy_stream: TcpStream) -> Result<()>{

        let mut reader = BufReader::new(stream);

        loop{

            let mut line = String::new();

            match reader.read_line(&mut line){

                Err(error) => return Err(error),

                Ok(0) => {
                    println!("Browser connection closed.");
                    break;
                }

                Ok(_bytes_received) => {

                    // last line was read
                    if line.trim().is_empty(){
                        continue;
                    }

                    if let Some(uri) = Self::extract_uri(line){

                        let file_path =Self::get_file_path(&uri);
                        let file_path = Path::new(&file_path);

                        if file_path.exists(){


                        }

                    }

                }

            }


        }

        Ok(())
    }

    pub fn sender_tcp_thread(){

    }

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

    pub fn get_file_path(uri: &str) -> String{
        return format!("{}/{}", CACHE_PATH, encode_path(uri));
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