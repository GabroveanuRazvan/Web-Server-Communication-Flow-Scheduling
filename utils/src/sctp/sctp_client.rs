use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use libc::{IPPROTO_SCTP, MSG_DONTWAIT, MSG_PEEK, SCTP_EVENTS, SCTP_INITMSG, SCTP_STATUS};
use crate::libc_wrappers::{safe_close, safe_dup, safe_getsockopt, safe_recv, safe_setsockopt, CStruct, SockAddrIn};
use crate::sctp::sctp_api::{events_to_u8, events_to_u8_mut, safe_sctp_connectx, safe_sctp_recvmsg, safe_sctp_sendmsg, safe_sctp_socket, SctpEventSubscribe, SctpInitMsg, SctpPeerBuilder, SctpSenderReceiveInfo, SctpStatus};
use io::Result;
use std::os::fd::RawFd;

#[derive(Debug)]
pub struct SctpStream{
    sock_fd: RawFd,
    // this will be assigned if the stream was created by an accept call or by a connect call
    address: SocketAddrV4,
    // this will be assigned when the stream calls connect
    peer_addresses: Option<Vec<Ipv4Addr>>,
    // if the stream is created by accept this will be None
    active_events: Option<SctpEventSubscribe>,
    ttl: u32,
}

impl SctpStream{

    pub fn new(sock_fd: i32, address: SocketAddrV4) -> Self{

        Self{
            sock_fd,
            address,
            peer_addresses: None,
            active_events: None,
            ttl: 0,
        }
    }

    pub fn connect(&mut self) -> &Self{

        // // crate a new socket
        // match safe_sctp_socket(){
        //     Ok(sock_fd) => self.sock_fd = sock_fd,
        //     Err(error)=> panic!("Sctp stream socket error: {}",error),
        // }

        // check if we have any addresses to connect to
        let peer_addresses = match self.peer_addresses{
            Some(ref addresses) => addresses,
            None => panic!("Sctp stream peer addresses is None while using connect"),
        };

        let mut socket_addresses: Vec<SockAddrIn> = Vec::new();

        // convert the ivp4 peer addresses to C sockaddr_in
        for address in peer_addresses{

            let current_socket_address = SockAddrIn::from_ipv4(self.address.port(),address.clone());
            println!("{:?}",current_socket_address);

            socket_addresses.push(current_socket_address)

        }

        if let Err(error) = safe_sctp_connectx(self.sock_fd, &mut socket_addresses){
            panic!("Connect error: {}", error);
        }

        self
    }

    /// Method used to set write ttl
    pub fn set_ttl(&mut self, ttl: u32) -> &Self{
        self.ttl = ttl;
        self
    }

    /// Method used to get ttl
    pub fn ttl(&self) ->u32{
        self.ttl
    }

    /// Method used to get the local address of the stream that was returned by accept
    pub fn local_address(&self) -> SocketAddrV4{
        self.address.clone()
    }

    /// Method used to read data from the socket, stores the client address and info
    pub fn read(&self, buffer: &mut [u8],
                sender_info: Option<&mut SctpSenderReceiveInfo>,
                flags: Option<&mut i32>) ->Result<usize>{

        let mut returned_sock_addr_c = self.local_address().clone().into();

        let mut dummy_flags = 0;

        // if flags is None just pass the reference of dummyflags
        match safe_sctp_recvmsg(self.sock_fd, buffer, Some(&mut returned_sock_addr_c), sender_info, match flags{
            Some(flags) => flags,
            None => &mut dummy_flags,
        }){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }

    }

    /// Method used to write data to a peer using a designated stream
    pub fn write(&self, buffer: &[u8], num_bytes: usize, stream_number: u16, ppid: u32,context: u32) -> Result<usize>{

        let mut sock_addr_c = self.local_address().clone().into();

        match safe_sctp_sendmsg(self.sock_fd,buffer,num_bytes,&mut sock_addr_c,ppid,0,stream_number,self.ttl,context){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }

    }
    /// Method used to write all data to a peer using a designated stream
    pub fn write_all(&self, buffer: &[u8], stream_number: u16, ppid: u32,context: u32) -> Result<usize>{
        let num_bytes = buffer.len();

        let mut sock_addr_c = self.local_address().clone().into();

        match safe_sctp_sendmsg(self.sock_fd,buffer,num_bytes,&mut sock_addr_c,ppid,0,stream_number,self.ttl,context){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }
    }

    /// Method used to write the buffer in a loop using chunks of chunk_size bytes
    pub fn write_chunked(&self, buffer: &[u8],chunk_size: usize, stream_number: u16, ppid: u32,context: u32)-> Result<usize>{
        let mut sock_addr_c: SockAddrIn = self.local_address().clone().into();

        let mut total_bytes = 0usize;

        for chunk in buffer.chunks(chunk_size) {
            total_bytes += self.write_all(chunk,stream_number,ppid,context)?;
        }

        Ok(total_bytes)

    }

    /// Method used to write one null terminated message to mark end of writing/reading to fellow peer
    pub fn write_null(&self,stream_number: u16,ppid: u32,context: u32) -> Result<usize>{

        let buffer: [u8;5] = [0;5];

        let mut sock_addr_c = self.local_address().clone().into();

        match safe_sctp_sendmsg(self.sock_fd,&buffer,1,&mut sock_addr_c,ppid,0,stream_number,self.ttl,context){
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
    /// !!! Should always be called AFTER connect call
    pub fn events(&self) ->&Self{

        let events_ref = match &self.active_events {
            Some(events) => events_to_u8(events),
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

        if let Err(error) = safe_getsockopt(self.sock_fd,IPPROTO_SCTP,SCTP_EVENTS,events_to_u8_mut(&mut events)){
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
            address: self.address.clone(),
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
    // this will be completed if the stream was created by an accept call or be the first peer address if the client connects
    address: SocketAddrV4,
    // this will be completed if the stream calls connect
    peer_addresses: Option<Vec<Ipv4Addr>>,
    // if the stream is created by accept this will be None
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
            address: self.address,
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
            address: SocketAddrV4::new(Ipv4Addr::new(0,0,0,0),0),
            peer_addresses: None,
            active_events: None,
            incoming_stream_count: 10,
            outgoing_stream_count: 10,
            ttl: 0,
        }
    }

    /// Creates a new stream like sctp socket
    fn socket(mut self) -> Self{

        let result = safe_sctp_socket();

        match result{
            Ok(descriptor) => self.sock_fd = descriptor,
            Err(e) => panic!("Sctp socket error: {e}"),
        };

        self
    }

    /// Adds the main address that the client will use to read and write data
    fn address(mut self,ipv4: Ipv4Addr) -> Self{

        self.address.set_ip(ipv4);
        self
    }

    /// Adds a subset of addresses to be later connected to
    fn addresses(mut self, mut addresses: Vec<Ipv4Addr>) -> Self{

        self.peer_addresses = Some(addresses);
        self
    }

    /// Sets the port of where the clinet will connect to
    fn port(mut self,port: u16) -> Self{

        self.address.set_port(port);
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

#[derive(Debug)]
pub struct SctpPacketData{
    pub stream_number: u16,
    pub ppid: u32,
    pub flags: u32,
    pub context:u32,
}

impl SctpPacketData{
    pub fn new(stream_number: u16, flags: u32, ppid: u32, context: u32) -> Self{

        Self{
            stream_number,
            flags,
            ppid,
            context,
        }
    }

}