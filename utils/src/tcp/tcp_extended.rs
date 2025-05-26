use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::io::{Error, ErrorKind, Read, Result};
use libc::{SOL_SOCKET, SO_RCVBUF, SO_SNDBUF};
use crate::libc_wrappers::{safe_getsockopt, safe_setsockopt, SocketBuffers};

impl SocketBuffers for TcpStream{
    fn set_send_buffer_size(&self,buffer_size: usize) -> std::io::Result<i32> {
        let fd = self.as_raw_fd();
        safe_setsockopt(fd,SOL_SOCKET,SO_SNDBUF,&buffer_size.to_le_bytes())
    }

    fn set_receive_buffer_size(&self,buffer_size: usize) -> std::io::Result<i32> {
        let fd = self.as_raw_fd();
        safe_setsockopt(fd,SOL_SOCKET,SO_RCVBUF,&buffer_size.to_le_bytes())
    }

    fn get_send_buffer_size(&self) -> std::io::Result<usize> {
        let fd = self.as_raw_fd();
        let mut num_bytes = [0u8;8];

        safe_getsockopt(fd,SOL_SOCKET,SO_SNDBUF,&mut num_bytes).map(|_|{
            usize::from_le_bytes(num_bytes)
        })
    }

    fn get_receive_buffer_size(&self) -> std::io::Result<usize> {
        let fd = self.as_raw_fd();
        let mut num_bytes = [0u8;8];

        safe_getsockopt(fd,SOL_SOCKET,SO_RCVBUF,&mut num_bytes).map(|_|{
            usize::from_le_bytes(num_bytes)
        })
    }
}


pub trait HtmlReadable{
    fn receive_header(&mut self,buffer: &mut [u8]) -> Result<usize>;
}

impl HtmlReadable for TcpStream{
    fn receive_header(&mut self,buffer: &mut [u8]) -> Result<usize>{
        
        let mut bytes_received: Option<usize> = None;
        let buffer_size = buffer.len();
        
        // Peek into the buffer while the end delimiter is not found
        while !buffer.windows(4).any(|w| w == b"\r\n\r\n"){
            let current_bytes = self.peek(buffer)?;
            
            if current_bytes == 0{
                return  Err(Error::new(ErrorKind::UnexpectedEof, "EOF"));
            }
            
            // If the buffer is full and the delimiter is not found return an error
            if let Some(bytes) = bytes_received{
                
                if current_bytes == bytes && bytes == buffer_size{
                    return Err(Error::new(ErrorKind::InvalidData, "Invalid data or buffer size too small"));
                }
                
            }
            
            bytes_received = Some(current_bytes);
        }
        
        // Find the end position of the header
        let header_end = buffer.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
        
        // Read just enough bytes for the header
        self.read_exact(&mut buffer[..header_end])?;
        
        Ok(header_end)
    }
}


#[cfg(test)]
pub mod tests {
    use std::io::Write;
    use std::net::TcpListener;
    use std::sync::{mpsc};
    use std::thread;
    use crate::constants::KILOBYTE;
    use super::*;
    
    #[test]
    pub fn tcp_stream_read_html_header1(){
        
        let header = String::from("GET /test HTTP/1.1\r\nHost: test\r\n\r\n");
        let residue = String::from("ana are mere");
        
        let request = header.clone() + &residue;
        
        let (tx,rx) = mpsc::channel();
        
        let server_thread = thread::spawn(move || {
            
            let mut port = 7878;
            
            let listener = loop {
                
                match TcpListener::bind(("0.0.0.0", port)){
                    Ok(server) => break server,
                    _ => {
                        port += 1;
                        continue;
                    }
                };
                
            };

            tx.send(port).unwrap();
            
            let (mut client,_addr) = listener.accept().unwrap();
            
            client.write(request.as_bytes()).unwrap();
            
        });
        
        let conn_thread = thread::spawn(move || {
            
            let port = rx.recv().unwrap();
            
            let mut client = TcpStream::connect(("127.0.0.1",port)).unwrap();
            
            let mut buffer = [0u8;4 * KILOBYTE];
            
            let header_bytes = client.receive_header(&mut buffer).unwrap();
            
            assert_eq!(header_bytes, header.len());
            
            let residue_bytes = client.read(&mut buffer).unwrap();
            
            assert_eq!(residue_bytes, residue.len());
            
            
        });
        
        server_thread.join().unwrap();
        conn_thread.join().unwrap();
        
    }
    
    
    
}