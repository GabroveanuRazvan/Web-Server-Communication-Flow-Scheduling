extern crate libc;

use std::ffi::c_void;
use libc::{c_int, size_t, sockaddr_in, socklen_t, sctp_sndrcvinfo, sctp_assoc_t, socket, AF_INET, SOCK_SEQPACKET, IPPROTO_SCTP, SOCK_STREAM};

use std::ptr;
use std::io::{Result};
use super::libc_wrappers::{wrap_result_nonnegative,SockAddrIn};


/// Macros used in sctp_bindx function
pub const SCTP_BINDX_ADD_ADDR: c_int = 1;
pub const SCTP_BINDX_REM_ADDR: c_int = 2;

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
    let sender_info_data = if let Some(info) = sender_info{
        info as *mut sctp_sndrcvinfo
    }
    else{
        ptr::null_mut()
    };

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

pub fn safe_sctp_bindx(socket_fd: i32, addrs: &mut [SockAddrIn], flags: i32) -> Result<i32>{
    let address_count = addrs.len() as i32;
    let addrs_ptr = addrs.as_mut_ptr() as *mut sockaddr_in;

    let result = unsafe{
        sctp_bindx(socket_fd,addrs_ptr,address_count,flags)
    };

    wrap_result_nonnegative(result)

}

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
        socket(AF_INET,SOCK_SEQPACKET,IPPROTO_SCTP)
    };

    wrap_result_nonnegative(result)

}

