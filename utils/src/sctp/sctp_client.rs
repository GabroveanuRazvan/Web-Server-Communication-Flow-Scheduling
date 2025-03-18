use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use libc::{IPPROTO_SCTP, MSG_DONTWAIT, MSG_PEEK, SCTP_EVENTS, SCTP_INITMSG, SCTP_STATUS};
use crate::libc_wrappers::{safe_close, safe_dup, safe_getsockopt, safe_recv, safe_setsockopt, CStruct, SockAddrIn};
use crate::sctp::sctp_api::{safe_sctp_connectx, safe_sctp_recvmsg, safe_sctp_sendmsg, safe_sctp_socket, SctpEventSubscribe, SctpInitMsg, SctpPeerBuilder, SctpSenderReceiveInfo, SctpStatus};
use io::Result;
use std::os::fd::RawFd;

#[derive(Debug)]
pub struct SctpStream{
    sock_fd: RawFd,
    port: u16,
    // this will be assigned when the stream calls connect
    peer_addresses: Vec<Ipv4Addr>,
    // if the stream is created by accept this will be None
    active_events: Option<SctpEventSubscribe>,
    ttl: u32,
}

impl SctpStream{

    pub(crate) unsafe fn from(sock_fd: i32,address: SocketAddrV4) -> Self{

        Self{
            sock_fd,
            port: address.port(),
            peer_addresses: vec![address.ip().clone()],
            active_events: None,
            ttl: 0,
        }
    }

    pub fn connect(&mut self) -> &Self{

        // Check if we have any addresses to connect to
        if self.peer_addresses.is_empty(){
            panic!("No addresses to connect to were provided");
        }

        let mut socket_addresses: Vec<SockAddrIn> = Vec::with_capacity(self.peer_addresses.len());

        // convert the ivp4 peer addresses to C sockaddr_in
        for address in &self.peer_addresses{

            let current_socket_address = SockAddrIn::from_ipv4(self.port,address.clone());
            socket_addresses.push(current_socket_address)

        }

        if let Err(error) = safe_sctp_connectx(self.sock_fd, &mut socket_addresses){
            panic!("Connect error: {}", error);
        }

        self
    }

    /// Sets the ttl.
    pub fn set_ttl(&mut self, ttl: u32) -> &Self{
        self.ttl = ttl;
        self
    }

    /// Gets the ttl.
    pub fn ttl(&self) ->u32{
        self.ttl
    }

    /// Gets a slice to the local ipv4 addresses of the stream.
    pub fn local_addresses(&self) -> &[Ipv4Addr]{
        self.peer_addresses.as_slice()
    }

    /// Returns a socket address, having the first ip of the peer addresses.
    pub fn local_address(&self) -> SocketAddrV4{
        SocketAddrV4::new(self.peer_addresses[0].clone(),self.port)
    }

    /// Method used to read data from the socket, stores the client address and info
    pub fn read(&self, buffer: &mut [u8],
                sender_info: Option<&mut SctpSenderReceiveInfo>,
                flags: Option<&mut i32>) ->Result<usize>{

        // let mut returned_sock_addr_c = self.local_address().into();

        let mut dummy_flags = 0;

        // if flags is None just pass the reference of dummyflags
        match safe_sctp_recvmsg(self.sock_fd, buffer, None, sender_info, match flags{
            Some(flags) => flags,
            None => &mut dummy_flags,
        }){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }

    }

    /// Method used to write data to a peer using a designated stream
    pub fn write(&self, buffer: &[u8], num_bytes: usize, stream_number: u16, ppid: u32,context: u32) -> Result<usize>{

        match safe_sctp_sendmsg(self.sock_fd,buffer,num_bytes,ppid,0,stream_number,self.ttl,context){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }

    }
    /// Method used to write all data to a peer using a designated stream
    pub fn write_all(&self, buffer: &[u8], stream_number: u16, ppid: u32,context: u32) -> Result<usize>{
        let num_bytes = buffer.len();

        match safe_sctp_sendmsg(self.sock_fd,buffer,num_bytes,ppid,0,stream_number,self.ttl,context){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }
    }

    /// Method used to peek into the socket buffer
    pub fn peek(&self, buffer: &mut[u8]) -> Result<usize>{

        let message_size = buffer.len();

        match safe_recv(self.sock_fd,buffer,message_size,MSG_PEEK | MSG_DONTWAIT){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }
    }

    /// Method used to activate the event options of the client
    /// !!! Must always be called AFTER connect call
    pub fn events(&self) ->&Self{

        let events_ref = match &self.active_events {
            Some(events) => events.as_bytes(),
            None => panic!("No events were specified"),
        };

        if let Err(error) = safe_setsockopt(self.sock_fd,IPPROTO_SCTP,SCTP_EVENTS,events_ref){
            panic!("SCTP setsockopt error: {error}");
        }

        self
    }

    /// Method used to get the active events of the client
    pub fn get_events(&self) -> SctpEventSubscribe{
        let mut events = SctpEventSubscribe::new();

        if let Err(error) = safe_getsockopt(self.sock_fd,IPPROTO_SCTP,SCTP_EVENTS,events.as_mut_bytes()){
            panic!("SCTP getsockopt error: {error}");
        }

        events
    }

    pub fn get_sctp_status(&self) -> SctpStatus{
        let mut sctp_status = SctpStatus::new();

        if let Err(error) = safe_getsockopt(self.sock_fd,IPPROTO_SCTP,SCTP_STATUS,sctp_status.as_mut_bytes()){
            panic!("SCTP getsockopt error: {error}");
        }

        sctp_status
    }

    /// Tries to clone the current stream by creating a new file descriptor for the current socket.
    pub fn try_clone(&self) -> Result<Self>{

        let new_sock_fd = safe_dup(self.sock_fd)?;

        Ok(Self{
            sock_fd: new_sock_fd,
            port: self.port,
            peer_addresses: self.peer_addresses.clone(),
            active_events: self.active_events.clone(),
            ttl: self.ttl,

        })

    }

}

/// Used to gracefully close the socket descriptor when the client goes out of scope
impl Drop for SctpStream{
    fn drop(&mut self){

        match safe_close(self.sock_fd){
            Ok(_) =>  println!("Sctp stream closed"),
            Err(error) => panic!("Server closed unexpectedly: {error}")
        }

    }

}


/// Builder pattern for sctp stream used when the stream acts as a client that will call connect

pub struct SctpStreamBuilder{
    sock_fd: i32,

    port: u16,
    peer_addresses: Vec<Ipv4Addr>,

    // when the stream is created by accept this will be None
    active_events: Option<SctpEventSubscribe>,

    outgoing_stream_count: u16,
    incoming_stream_count: u16,

    ttl: u32,
}

impl SctpStreamBuilder{

    /// Sets the ttl
    pub fn ttl(mut self, ttl: u32)->Self{
        self.ttl = ttl;
        self
    }

    /// Builds the client based on the given information
    pub fn build(self) -> SctpStream{

        let mut sctp_init = SctpInitMsg::new();
        sctp_init.sinit_num_ostreams = self.outgoing_stream_count;
        sctp_init.sinit_max_instreams = self.incoming_stream_count;

        if let Err(error) = safe_setsockopt(self.sock_fd,IPPROTO_SCTP,SCTP_INITMSG,sctp_init.as_mut_bytes()){
            panic!("SCTP setsockopt error: {error}");
        }

        SctpStream{
            sock_fd: self.sock_fd,
            port: self.port,
            peer_addresses: self.peer_addresses,
            active_events: self.active_events,
            ttl: self.ttl,
        }
    }
}

impl SctpPeerBuilder for SctpStreamBuilder {

    /// Creates a new builder with default values
    fn new() -> Self{

        Self{
            sock_fd: 0,
            peer_addresses: Vec::new(),
            port: 0,
            active_events: None,
            incoming_stream_count: 10,
            outgoing_stream_count: 10,
            ttl: 0,
        }
    }

    /// Creates a new one to one sctp socket.
    fn socket(mut self) -> Self{

        let result = safe_sctp_socket();

        match result{
            Ok(descriptor) => self.sock_fd = descriptor,
            Err(e) => panic!("Sctp socket error: {e}"),
        };

        self
    }

    /// Adds an address to the peer addresses.
    fn address(mut self,ipv4: Ipv4Addr) -> Self{

        self.peer_addresses.push(ipv4);
        self
    }

    /// Adds a subset of addresses to be later connected to.
    fn addresses(mut self, mut addresses: Vec<Ipv4Addr>) -> Self{

        self.peer_addresses.append(&mut addresses);
        self
    }

    /// Sets the port.
    fn port(mut self,port: u16) -> Self{

        self.port = port;
        self
    }

    /// Sets the events that the client will be subscribed to
    fn events(mut self, events: SctpEventSubscribe) -> Self{

        self.active_events = Some(events);
        self
    }

    /// Sets the maximum number of outgoing streams
    fn set_outgoing_streams(mut self, out_stream_count: u16) ->Self{

        self.outgoing_stream_count = out_stream_count;
        self
    }

    /// Sets the maximum number of incoming streams
    fn set_incoming_streams(mut self, in_stream_count: u16) ->Self{

        self.incoming_stream_count = in_stream_count;
        self
    }

}