use std::net::{SocketAddr, ToSocketAddrs,TcpStream,TcpListener};
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::num::NonZeroU8;
use crate::constants::{MAX_MESSAGE_SIZE, TCP_ASSOC_META_SIZE};
use crate::packets::byte_packet::BytePacket;

pub struct TcpAssociation {
    stream_count: NonZeroU8,
    control_stream: TcpStream,
    streams: Vec<TcpStream>,
}

impl TcpAssociation{
    
    /// Creates a new TcpAssociation.
    fn new(stream_count: u8,control_stream: TcpStream) -> Self{
        Self{
            stream_count: NonZeroU8::new(stream_count).unwrap_or(unsafe{NonZeroU8::new_unchecked(1)}),
            control_stream,
            streams: Vec::with_capacity(stream_count as usize),
        }
    }

    /// Connects to an association. Before connecting the peers will exchange the number of streams of the association through the command stream.
    pub fn connect(to: impl ToSocketAddrs,stream_count: u8) -> Result<Self>{

        let mut buffer = [0u8;1];

        // Begin the stream count exchange
        let mut control_stream = TcpStream::connect(&to)?;

        // Sent this assoc chosen number of streams
        control_stream.write_all(stream_count.to_be_bytes().as_slice())?;
        control_stream.flush()?;

        // Receive the peer stream count
        control_stream.read_exact(&mut buffer)?;
        let peer_stream_count = u8::from_be_bytes(buffer);

        // Compute the minimum
        let final_stream_count = peer_stream_count.min(stream_count);

        // Wait for the peer to send anything to mark it is ready to receive connections
        match control_stream.read(&mut buffer){
            Ok(0) => return Err(Error::new(ErrorKind::UnexpectedEof, "Control stream closed")),
            Ok(n) => (),
            Err(err) => return Err(err),
        }

        let mut assoc = Self::new(final_stream_count, control_stream);
        for _ in 0..assoc.stream_count.get() {
            assoc.streams.push(TcpStream::connect(&to)?);
        }


        Ok(assoc)
    }

    /// Stream count getter.
    pub fn stream_count(&self) -> u8{
        self.stream_count.get()
    }

    /// Clones the association using try_clone call on each stream.
    pub fn try_clone(&self) -> Result<Self>{

        let mut new_streams = Vec::with_capacity(self.stream_count.get() as usize);

        for stream in &self.streams{
            new_streams.push(stream.try_clone()?);
        }

        let mut assoc = Self::new(self.stream_count.get(),self.control_stream.try_clone()?);
        assoc.streams = new_streams;

        Ok(assoc)

    }

    /// Sends a message through a stream.
    /// The message will have a ppid attached to its payload.
    /// The stream number will be sent through the command stream.
    pub fn send(&mut self,message: &[u8], stream: usize, ppid: u32) -> Result<()>{

        // Check for errors
        let message_size = message.len();
        if message_size > MAX_MESSAGE_SIZE {
            return Err(Error::new(ErrorKind::Other, "Message too big"));
        }

        if stream > self.stream_count.get() as usize{
            return Err(Error::new(ErrorKind::Other, "Invalid stream index"));
        }

        // Build the packet that will be sent: ppid + message size + message
        let packet_size = TCP_ASSOC_META_SIZE + message_size;
        
        let mut byte_packet = BytePacket::new(packet_size);

        byte_packet.write_u32(ppid).map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;
        byte_packet.write_u64(message_size as u64).map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;
        unsafe{
            byte_packet.write_buffer(message).map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;
        }

        // Send the packet on the designated stream, and the stream index on the control stream
        self.streams[stream].write_all(byte_packet.get_buffer())?;
        self.control_stream.write_all((stream as u8).to_be_bytes().as_slice())?;

        Ok(())

    }

    /// Receives a message from a stream.
    /// Reads the stream number from the command control, so that the peer can know from what stream to read.
    pub fn receive(&mut self) -> Result<AssocMessage>{
        
        // Get the stream index from the control connection
        let mut stream_buf = [0u8;1];
        self.control_stream.read_exact(&mut stream_buf)?;

        let stream = stream_buf[0] as usize;
        
        // Read the medata into a buffer
        let mut meta_buffer = [0u8;TCP_ASSOC_META_SIZE];
        self.streams[stream].read_exact(&mut meta_buffer)?;

        // Parse the metadata packet
        let mut packet = BytePacket::from(&meta_buffer);
        let ppid = packet.read_u32().map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;
        let message_size = packet.read_u64().map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;
        
        // Receive the message
        let mut message = vec![0u8; message_size as usize];
        self.streams[stream].read_exact(&mut message)?;

        Ok(AssocMessage::new(message,stream_buf[0],ppid))

    }
    
    pub fn peer_addresses(&self) -> Vec<SocketAddr>{
        let mut addresses : Vec<SocketAddr> = self.streams.iter().map(|stream| stream.peer_addr().unwrap()).collect();
        addresses.push(self.control_stream.peer_addr().unwrap());
        addresses
        
    }

}

#[derive(Debug)]
pub struct AssocMessage{
    pub message: Vec<u8>,
    pub stream: u8,
    pub ppid: u32,
}

impl AssocMessage{
    /// Create a new tcp association message.
    fn new(message: Vec<u8>, stream: u8, ppid: u32) -> Self{
        Self{
            message,
            stream,
            ppid,
        }
    }
}

/// Tcp Association Sever
pub struct TcpAssociationListener{
    stream_count: u8,
    listener: TcpListener,
}

impl TcpAssociationListener{
    
    /// Binds to a given address of fixed stream count.
    pub fn bind(address: impl ToSocketAddrs,stream_count: u8) -> Result<Self>{
        Ok(
            Self{
                stream_count,
                listener: TcpListener::bind(address)?,
            }
        )
    }

    /// Accepts a new association. Exchanges the stream count with its peer.
    pub fn accept(&self) -> Result<(TcpAssociation,Vec<SocketAddr>)>{

        let mut buffer = [0u8;1];

        // Wait to receive the exchange connection
        let (mut control_stream,addr) = self.listener.accept()?;

        // Send the maximum number of streams accepted
        control_stream.write_all(self.stream_count.to_be_bytes().as_slice())?;
        control_stream.flush()?;

        // Get the peer max number of streams
        control_stream.read_exact(&mut buffer)?;
        let peer_stream_count = u8::from_be_bytes(buffer);

        // Compute the minimum
        let final_stream_count = peer_stream_count.min(self.stream_count);
        let mut addresses = Vec::with_capacity(final_stream_count as usize);

        // Send something to mark as ready
        control_stream.write_all(b"R")?;

        // Create the association and accept the incoming connections
        let mut assoc = TcpAssociation::new(final_stream_count, control_stream);
        for _ in 0..assoc.stream_count.get(){
            let (stream,addr) = self.listener.accept()?;
            assoc.streams.push(stream);
            addresses.push(addr);
        }

        Ok((assoc,addresses))

    }

    /// Stream count getter.
    pub fn stream_count(&self) -> u8{
        self.stream_count
    }
    
    pub fn local_addr(&self) -> Result<SocketAddr>{
        self.listener.local_addr()
    }
    
    pub fn incoming(&self) -> Incoming{
        Incoming::new(self)
    }
}

pub struct Incoming<'a>{
    assoc_listener: &'a TcpAssociationListener,
}

impl<'a> Incoming<'a>{
    pub fn new(assoc_listener: &'a TcpAssociationListener) -> Self{
        Self{
            assoc_listener,
        }
    }
}

impl<'a> Iterator for Incoming<'a>{
    
    type Item = Result<TcpAssociation>;
    
    /// Accept a request or return None if accept fails.
    fn next(&mut self) -> Option<Self::Item> {
        match self.assoc_listener.accept(){
            Ok((assoc,_addr)) => Some(Ok(assoc)),
            Err(error) => Some(Err(error)),
        }
    }
}