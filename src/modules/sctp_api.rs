extern crate libc;

use libc::{c_int, c_void, size_t, sockaddr_in, socklen_t, sctp_sndrcvinfo, sctp_assoc_t, socket, AF_INET, SOCK_SEQPACKET, IPPROTO_SCTP, SOCK_STREAM, SOCK_DGRAM};

use std::{ptr, slice};
use std::io::{Result};
use super::libc_wrappers::{debug_sctp_sndrcvinfo, get_ptr_from_mut_ref, wrap_result_nonnegative, SctpSenderInfo, SockAddrIn};


/// Macros used in sctp_bindx function
pub const SCTP_BINDX_ADD_ADDR: c_int = 1;
pub const SCTP_BINDX_REM_ADDR: c_int = 2;

///
/// Custom structs
///


/// Same SctpEventSubscribe as in the C API
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


impl SctpEventSubscribe{
    /// Method used to quickly initialize a raw object without using mem::zeroed
    pub fn new() -> SctpEventSubscribe {
        SctpEventSubscribe{
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
}


/// FFI binding of sctp functions that the libc crate does not provide
#[link(name = "sctp")]
extern "C"{
    fn sctp_recvmsg(sd: c_int, msg: *mut c_void, len: size_t, from: *mut sockaddr_in, fromlen: *mut socklen_t, sri: *mut sctp_sndrcvinfo, msg_flags: *mut c_int) -> c_int;
    fn sctp_sendmsg(sd: c_int, msg: *const c_void, len: size_t, to: *const sockaddr_in, tolen: socklen_t, ppid: u32, flags: u32, stream_no: u16, timetolive: u32, context: u32) -> c_int;
    fn sctp_bindx(sd: c_int, addrs: *mut sockaddr_in, addrcnt: c_int, flags: c_int) -> c_int;
    fn sctp_connectx(sd: c_int, addrs: *mut sockaddr_in, addrcnt: c_int, flags: c_int) -> c_int;
    fn sctp_getpaddrs(sd: c_int, assoc_id: sctp_assoc_t, addrs: *mut *mut sockaddr_in) -> c_int;
    fn sctp_freepaddrs(addrs: *mut sockaddr_in);
    fn sctp_getladdrs(sd: c_int, assoc_id: sctp_assoc_t, addrs: *mut *mut sockaddr_in) -> c_int;
    fn sctp_freeladdrs(addrs: *mut sockaddr_in);
    fn sctp_opt_info(sd: c_int,assoc_id: sctp_assoc_t, opt: c_int, arg: *mut c_void, size: *mut socklen_t) -> c_int;
    fn sctp_peeloff(sd: c_int,assoc_id: sctp_assoc_t) -> c_int;

}


/// Wrapper for sctp_recv function
pub fn safe_sctp_recvmsg(

    sock_fd: i32,
    msg: &mut [u8],
    from_address: Option<&mut SockAddrIn>,
    sender_info: Option<&mut sctp_sndrcvinfo>,
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

    let sender_info_data = get_ptr_from_mut_ref(sender_info);

    // call the unsafe FFI
    let result = unsafe{

        sctp_recvmsg(sock_fd,
                     msg.as_mut_ptr() as *mut c_void,
                     message_size,
                     from_address_data.0,
                     from_address_data.1,
                     sender_info_data,
                     msg_flags as *mut c_int
        )

    };

    // return Ok or Err based on the output
    wrap_result_nonnegative(result)
}

/// Wrapper function for sctp_sendmsg
pub fn safe_sctp_sendmsg(
    sock_fd: i32,
    msg: &[u8],
    to_address: &mut SockAddrIn,
    payload_protocol_id: u32,
    flags: u32,
    stream_number: u16,
    time_to_live: u32,
    context: u32

) -> Result<i32> {

    // get the sizes of message and address
    let message_size = msg.len() as size_t;
    let address_size = size_of::<SockAddrIn>() as socklen_t;

    // call the unsafe FFI
    let result = unsafe{

        sctp_sendmsg(sock_fd,
                     msg.as_ptr() as *const c_void,
                     message_size,
                     to_address,
                     address_size,
                     payload_protocol_id,
                     flags,
                     stream_number,
                     time_to_live,
                     context
        )

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
pub fn safe_sctp_connectx(socket_fd: i32,addrs: &mut [SockAddrIn], flags: i32) -> Result<i32>{
    let address_count = addrs.len() as i32;
    let addrs_ptr = addrs.as_mut_ptr();

    let result = unsafe{
        sctp_connectx(socket_fd,addrs_ptr,address_count,flags)
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
pub fn safe_sctp_socket() -> Result<i32>{

    let result = unsafe{
        socket(AF_INET,SOCK_DGRAM,IPPROTO_SCTP)
    };

    wrap_result_nonnegative(result)

}


///
/// Custom structs related functions
///


/// Function that takes the address of the struct SctpEventSubscribe and turns the address into &[u8]
pub fn events_to_u8(events: &SctpEventSubscribe) -> &[u8]{

    let ptr = events as *const SctpEventSubscribe as *const u8;
    let size = size_of::<SctpEventSubscribe>();

    unsafe{slice::from_raw_parts(ptr, size)}
}
