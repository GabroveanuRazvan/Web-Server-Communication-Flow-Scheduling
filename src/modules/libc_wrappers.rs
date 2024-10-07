use std::ffi::CString;
use std::fmt;
use std::io::Error;
use libc::{__errno_location, c_int, listen, c_char, c_void, sockaddr_in, AF_INET, sctp_sndrcvinfo};
use std::io::Result;
use std::net::Ipv4Addr;

/// Aliases and structures that are not in libc

pub type SockAddrIn = sockaddr_in;
pub type SctpSenderInfo = sctp_sndrcvinfo;
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


extern "C"{
    fn inet_pton(af: c_int,src: *const c_char,dst: *mut c_void) -> c_int;
}

/// Wrapper for listen, returns Ok(0) or Err(io::Error) on failure
pub fn safe_listen(socket_fd: i32,max_queue_size: i32) -> Result<i32> {

    let result = unsafe{
        listen(socket_fd, max_queue_size)
    };

    wrap_result_nonnegative(result)

}

/// Wrapper for AF_INET inet_pton, returns Ok(0) or Err(io::Error) on failure
pub fn safe_inet_pton(ip: String, to: &mut u32) -> Result<i32>{

    let ip_as_cstring = CString::new(ip).unwrap();

    let result = unsafe{
        inet_pton(AF_INET, ip_as_cstring.as_ptr(), to as *mut u32 as *mut c_void)
    };

    wrap_result_positive(result)
}

/// Function that extracts errno safely
pub fn get_errno() -> i32{

    let mut errno = 0;

    unsafe{
        errno = *__errno_location();
    }

    errno

}

/// Wrapper function for nonnegative values
pub fn wrap_result_nonnegative(result: i32) -> Result<i32> {

    if result >= 0{
        Ok(result)
    }
    else{
        Err(Error::from_raw_os_error(get_errno()))
    }

}

/// Wrapper function for positive values
pub fn wrap_result_positive(result: i32) -> Result<i32> {

    if result > 0{
        Ok(result)
    }
    else{
        Err(Error::from_raw_os_error(get_errno()))
    }

}

/// Debugging functions
pub fn debug_sockaddr(sockaddr: &SockAddrIn){
    println!("Sockaddr(family:{}, port:{}, address: {})",sockaddr.sin_family,sockaddr.sin_port.to_be(),Ipv4Addr::from(sockaddr.sin_addr.s_addr.to_be()));
}
pub fn debug_sctp_sndrcvinfo(info: &SctpSenderInfo) {
    println!("SCTP Send/Receive Info:");
    println!("  Stream: {}", info.sinfo_stream);
    println!("  SSN: {}", info.sinfo_ssn);
    println!("  Flags: {}", info.sinfo_flags);
    println!("  PPID: {}", info.sinfo_ppid);
    println!("  Context: {}", info.sinfo_context);
    println!("  TSN: {}", info.sinfo_tsn);
    println!("  Cumulative TSN: {}", info.sinfo_cumtsn);
    println!("  Association ID: {}", info.sinfo_assoc_id);
}
