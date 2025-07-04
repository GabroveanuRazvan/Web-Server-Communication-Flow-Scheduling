extern crate libc;
use libc::{c_int, c_void, size_t, sockaddr_in, socklen_t, sctp_sndrcvinfo, sctp_assoc_t, socket, AF_INET, IPPROTO_SCTP, SOCK_STREAM};
use std::{fmt, ptr};
use std::io::{Result};
use std::net::{Ipv4Addr};
use std::os::fd::RawFd;
use crate::libc_wrappers::{CStruct, SockAddrStorage};
use super::super::libc_wrappers::{wrap_result_nonnegative, SockAddrIn};

/// ///////////
/// Macros ///
/// /////////

pub const SCTP_BINDX_ADD_ADDR: c_int = 1;
pub const SCTP_BINDX_REM_ADDR: c_int = 2;
pub const MAX_STREAM_NUMBER: u16 = 10;

/// /////////////////////////
/// Structures and traits ///
/// ////////////////////////

/// Builder pattern for sctp clients/servers
pub trait SctpPeerBuilder{
    fn new() -> Self;
    fn socket(self) -> Self;
    fn address(self,address: Ipv4Addr) -> Self;
    fn addresses(self,addresses: Vec<Ipv4Addr>) -> Self;
    fn port(self,port: u16) -> Self;
    fn events(self, events: SctpEventSubscribe) -> Self;
    fn set_outgoing_streams(self, out_stream_count: u16) ->Self;
    fn set_incoming_streams(self, in_stream_count: u16) ->Self;
}

#[repr(C,packed(4))]
#[derive(Copy, Clone)]
pub struct SctpPeerAddrInfo{
    pub spinfo_assoc_id: i32,
    pub sockaddr_storage: SockAddrStorage,
    pub spinfo_state: i32,
    pub spinfo_cwnd: u32,
    pub spinfo_srtt: u32,
    pub spinfo_rto: u32,
    pub spinfo_mtu: u32,
}
impl CStruct for SctpPeerAddrInfo{}

#[repr(C,packed(4))]
#[derive(Copy, Clone)]
pub struct SctpStatus{
    pub sstat_assoc_id: i32,
    pub sstat_state: i32,
    pub sstat_rwnd: u32,
    pub sstat_unackdata: u16,
    pub sstat_penddata: u16,
    pub sstat_instrms: u16,
    pub sstat_outstrms: u16,
    pub sstat_fragmentation_point: u32,
    pub sstat_primary: SctpPeerAddrInfo,
}
impl fmt::Debug for SctpStatus{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f, "SctpStatus{{InStreams: {}, OutStreams: {}}}", self.sstat_instrms, self.sstat_outstrms)

    }
}
impl CStruct for SctpStatus{}

#[repr(C)]
#[derive(Copy, Clone,Default,Debug)]
pub struct SctpInitMsg{
    pub sinit_num_ostreams: u16,
    pub sinit_max_instreams: u16,
    pub sinit_max_attempts: u16,
    pub sinit_max_init_timeo: u16,
}

impl CStruct for SctpInitMsg{}


#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct SctpSenderReceiveInfo{
    pub sinfo_stream: u16,
    pub sinfo_ssn: u16,
    pub sinfo_flags: u16,
    pub sinfo_ppid: u32,
    pub sinfo_context: u32,
    pub sinfo_timetolive: u32,
    pub sinfo_tsn: u32,
    pub sinfo_cumtsn: u32,
    pub sinfo_assoc_id: i32,
}

impl CStruct for SctpSenderReceiveInfo{}
impl SctpSenderReceiveInfo{
    pub fn as_mut_c_counterpart(&mut self) -> *mut sctp_sndrcvinfo{
        self as *mut Self as *mut sctp_sndrcvinfo
    }
    pub fn as_c_counterpart(&self) -> *const sctp_sndrcvinfo{
        self as *const Self as *const sctp_sndrcvinfo
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct SctpEventSubscribe {
    pub sctp_data_io_event: u8,
    pub sctp_association_event: u8,
    pub sctp_address_event: u8,
    pub sctp_send_failure_event: u8,
    pub sctp_peer_error_event: u8,
    pub sctp_shutdown_event: u8,
    pub sctp_partial_delivery_event: u8,
    pub sctp_adaptation_layer_event: u8,
    pub sctp_authentication_event: u8,
    pub sctp_sender_dry_event: u8,
    pub sctp_stream_reset_event: u8,
    pub sctp_assoc_reset_event: u8,
    pub sctp_stream_change_event: u8,
    pub sctp_send_failure_event_event: u8,
}

impl CStruct for SctpEventSubscribe{}

pub struct SctpEventSubscribeBuilder {
    sctp_data_io_event: u8,
    sctp_association_event: u8,
    sctp_address_event: u8,
    sctp_send_failure_event: u8,
    sctp_peer_error_event: u8,
    sctp_shutdown_event: u8,
    sctp_partial_delivery_event: u8,
    sctp_adaptation_layer_event: u8,
    sctp_authentication_event: u8,
    sctp_sender_dry_event: u8,
    sctp_stream_reset_event: u8,
    sctp_assoc_reset_event: u8,
    sctp_stream_change_event: u8,
    sctp_send_failure_event_event: u8,
}

impl SctpEventSubscribeBuilder {
    pub fn new() -> Self {
        Self {
            sctp_data_io_event: 0,
            sctp_association_event: 0,
            sctp_address_event: 0,
            sctp_send_failure_event: 0,
            sctp_peer_error_event: 0,
            sctp_shutdown_event: 0,
            sctp_partial_delivery_event: 0,
            sctp_adaptation_layer_event: 0,
            sctp_authentication_event: 0,
            sctp_sender_dry_event: 0,
            sctp_stream_reset_event: 0,
            sctp_assoc_reset_event: 0,
            sctp_stream_change_event: 0,
            sctp_send_failure_event_event: 0,
        }
    }

    pub fn sctp_data_io_event(mut self) -> Self {
        self.sctp_data_io_event = 1;
        self
    }

    pub fn sctp_association_event(mut self) -> Self {
        self.sctp_association_event = 1;
        self
    }

    pub fn sctp_address_event(mut self) -> Self {
        self.sctp_address_event = 1;
        self
    }

    pub fn sctp_send_failure_event(mut self) -> Self {
        self.sctp_send_failure_event = 1;
        self
    }

    pub fn sctp_peer_error_event(mut self) -> Self {
        self.sctp_peer_error_event = 1;
        self
    }

    pub fn sctp_shutdown_event(mut self) -> Self {
        self.sctp_shutdown_event = 1;
        self
    }

    pub fn sctp_partial_delivery_event(mut self) -> Self {
        self.sctp_partial_delivery_event = 1;
        self
    }

    pub fn sctp_adaptation_layer_event(mut self) -> Self {
        self.sctp_adaptation_layer_event = 1;
        self
    }

    pub fn sctp_authentication_event(mut self) -> Self {
        self.sctp_authentication_event = 1;
        self
    }

    pub fn sctp_sender_dry_event(mut self) -> Self {
        self.sctp_sender_dry_event = 1;
        self
    }

    pub fn sctp_stream_reset_event(mut self) -> Self {
        self.sctp_stream_reset_event = 1;
        self
    }

    pub fn sctp_assoc_reset_event(mut self) -> Self {
        self.sctp_assoc_reset_event = 1;
        self
    }

    pub fn sctp_stream_change_event(mut self) -> Self {
        self.sctp_stream_change_event = 1;
        self
    }

    pub fn sctp_send_failure_event_event(mut self) -> Self {
        self.sctp_send_failure_event_event = 1;
        self
    }

    pub fn build(self) -> SctpEventSubscribe {
        SctpEventSubscribe {
            sctp_data_io_event: self.sctp_data_io_event,
            sctp_association_event: self.sctp_association_event,
            sctp_address_event: self.sctp_address_event,
            sctp_send_failure_event: self.sctp_send_failure_event,
            sctp_peer_error_event: self.sctp_peer_error_event,
            sctp_shutdown_event: self.sctp_shutdown_event,
            sctp_partial_delivery_event: self.sctp_partial_delivery_event,
            sctp_adaptation_layer_event: self.sctp_adaptation_layer_event,
            sctp_authentication_event: self.sctp_authentication_event,
            sctp_sender_dry_event: self.sctp_sender_dry_event,
            sctp_stream_reset_event: self.sctp_stream_reset_event,
            sctp_assoc_reset_event: self.sctp_assoc_reset_event,
            sctp_stream_change_event: self.sctp_stream_change_event,
            sctp_send_failure_event_event: self.sctp_send_failure_event_event,
        }
    }
}


/// /////////////////////////////////
/// FFI bindings for the sctp API ///
/// ////////////////////////////////

#[link(name = "sctp")]
extern "C"{
    fn sctp_recvmsg(sd: c_int, msg: *mut c_void, len: size_t, from: *mut sockaddr_in, fromlen: *mut socklen_t, sri: *mut sctp_sndrcvinfo, msg_flags: *mut c_int) -> c_int;
    fn sctp_sendmsg(sd: c_int, msg: *const c_void, len: size_t, to: *const sockaddr_in, tolen: socklen_t, ppid: u32, flags: u32, stream_no: u16, timetolive: u32, context: u32) -> c_int;
    fn sctp_bindx(sd: c_int, addrs: *mut sockaddr_in, addrcnt: c_int, flags: c_int) -> c_int;
    fn sctp_connectx(sd: c_int, addrs: *mut sockaddr_in, addrcnt: c_int, flags: *mut c_int) -> c_int;
    fn sctp_getpaddrs(sd: c_int, assoc_id: sctp_assoc_t, addrs: *mut *mut sockaddr_in) -> c_int;
    fn sctp_freepaddrs(addrs: *mut sockaddr_in);
    fn sctp_getladdrs(sd: c_int, assoc_id: sctp_assoc_t, addrs: *mut *mut sockaddr_in) -> c_int;
    fn sctp_freeladdrs(addrs: *mut sockaddr_in);
    pub fn sctp_opt_info(sd: c_int,assoc_id: sctp_assoc_t, opt: c_int, arg: *mut c_void, size: *mut socklen_t) -> c_int;
    fn sctp_peeloff(sd: c_int,assoc_id: sctp_assoc_t) -> c_int;

}


/// Wrapper for sctp_recv function
pub fn safe_sctp_recvmsg(

    sock_fd: i32,
    msg: &mut [u8],
    from_address: Option<&mut SockAddrIn>,
    sender_info: Option<&mut SctpSenderReceiveInfo>,
    msg_flags: &mut i32

) -> Result<i32>{

    let message_size = msg.len() as size_t;

    // get a tuple of pointers to the socket address and its length or null pointers if they are not specified
    let from_address_data = if let Some(address) = from_address{

        let mut address_length = size_of::<SockAddrIn>() as socklen_t;

        (address as *mut SockAddrIn,&mut address_length as *mut socklen_t)
    }
    else{
        (ptr::null_mut(),ptr::null_mut())
    };

    // get the sender info if it was specified

    let sender_info_data = match sender_info{
        Some(info) => unsafe{info.as_mut_c_counterpart()},
        None => ptr::null_mut()
    };

    // call the unsafe FFI
    let result = unsafe{

        sctp_recvmsg(sock_fd,
                     msg.as_mut_ptr() as *mut c_void,
                     message_size,
                     from_address_data.0 as *mut sockaddr_in,
                     from_address_data.1,
                     sender_info_data,
                     msg_flags as *mut c_int
        )

    };

    // return Ok or Err based on the output
    wrap_result_nonnegative(result)
}

/// Wrapper function for sctp_sendmsg
/// !Does not use sockaddr_in, the function lets the protocol to decide the best association ipv4 to connect to
pub fn safe_sctp_sendmsg(
    sock_fd: i32,
    msg: &[u8],
    msg_size: usize,
    payload_protocol_id: u32,
    flags: u32,
    stream_number: u16,
    time_to_live: u32,
    context: u32

) -> Result<i32> {

    // get the sizes of message and address
    let message_size = msg_size as size_t;
    let address_size = size_of::<SockAddrIn>() as socklen_t;

    let result = match msg_size{
        0 => unsafe{
            sctp_sendmsg(sock_fd,
                         ptr::null() as *const c_void,
                         0,
                         ptr::null() as *const sockaddr_in,
                         0,
                         payload_protocol_id,
                         flags,
                         stream_number,
                         time_to_live,
                         context
            )
        }

        _msg_size => unsafe{
            sctp_sendmsg(sock_fd,
                         msg.as_ptr() as *const c_void,
                         message_size,
                         ptr::null() as *const sockaddr_in,
                         address_size,
                         payload_protocol_id,
                         flags,
                         stream_number,
                         time_to_live,
                         context
            )
        }
    };

    // return Ok or Err based on the output
    wrap_result_nonnegative(result)
}

/// Wrapper function for sctp_bindx function
pub fn safe_sctp_bindx(socket_fd: i32, addrs: &mut [SockAddrIn], flags: i32) -> Result<i32>{
    let address_count = addrs.len() as i32;
    let addrs_ptr = addrs.as_mut_ptr() as *mut sockaddr_in;

    let result = unsafe{
        sctp_bindx(socket_fd,addrs_ptr,address_count,flags)
    };

    wrap_result_nonnegative(result)

}

/// Wrapper function for sctp_connextx function
pub fn safe_sctp_connectx(socket_fd: i32,addrs: &mut [SockAddrIn]) -> Result<i32>{
    let address_count = addrs.len() as i32;
    let addrs_ptr = addrs.as_mut_ptr() as *mut sockaddr_in;

    let result = unsafe{
        sctp_connectx(socket_fd,addrs_ptr,address_count,ptr::null_mut())
    };

    wrap_result_nonnegative(result)

}

/// Wrapper for sctp_peeloff, returns Ok(0) or Err(io::Error) on failure
pub fn safe_sctp_peeloff(socket_fd: i32,assoc_id: i32 ) -> Result<i32>{

    let result = unsafe{
        sctp_peeloff(socket_fd,assoc_id)
    };

    wrap_result_nonnegative(result)

}

/// Creates an ipv4 sctp socket with delimited packets, returns Ok(socket_descriptor) or Err(io::Error) on failure
pub fn safe_sctp_socket() -> Result<RawFd>{

    let result = unsafe{
        socket(AF_INET,SOCK_STREAM,IPPROTO_SCTP)
    };

    wrap_result_nonnegative(result)

}