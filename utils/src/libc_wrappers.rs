use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::io::Error;
use libc::{__errno_location, c_int, listen, c_char, c_void, sockaddr_in, AF_INET, sctp_sndrcvinfo, setsockopt, accept, sockaddr, socklen_t, in_addr};
use std::io::Result;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::ptr;

/// Aliases and structures that are not in libc

pub type SockAddrIn = sockaddr_in;
pub type SctpSenderInfo = sctp_sndrcvinfo;

/// FFI bindings for functions that the libc crate does not provide
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

/// Wrapper for accept, returns Ok(0) or Err(io::Error) on failure
pub fn safe_accept(socket_fd: i32, address: Option<&mut SockAddrIn>, address_size: Option<&mut usize>) -> Result<i32>{


    let addr_ptr = get_ptr_from_mut_ref(address);
    let addr_size_ptr = get_ptr_from_mut_ref(address_size);

    let result = unsafe{
        accept(
            socket_fd,
            addr_ptr as *mut SockAddrIn as *mut sockaddr,
            addr_size_ptr as *mut socklen_t,
        )
    };

    wrap_result_nonnegative(result)

}

/// Wrapper for AF_INET inet_pton, returns Ok or Err(io::Error) on failure
pub fn safe_inet_pton(ip: String, to: &mut u32) -> Result<i32>{

    let ip_as_cstring = CString::new(ip).unwrap();

    let result = unsafe{
        inet_pton(AF_INET, ip_as_cstring.as_ptr(), to as *mut u32 as *mut c_void)
    };

    wrap_result_positive(result)
}

/// Wrapper function used to set the socket options
pub fn safe_setsockopt(socket: i32, level:i32, option_name:i32, option_value:&[u8]) -> Result<i32>{

    let option_length = option_value.len() as u32;

    let result = unsafe{
        setsockopt(socket,level,option_name,option_value.as_ptr() as *const c_void,option_length)
    };

    wrap_result_nonnegative(result)

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

/// Unwraps the option and returns a null pointer if None or a const pointer to type T otherwise
pub fn get_ptr_from_ref<T>(reference: Option<&T>) -> *const T{

    if let Some(address) = reference{
        address as *const T
    }
    else{
        ptr::null()
    }
}

/// Unwraps the option and returns a null pointer if None or a mutable pointer to type T otherwise
pub fn get_ptr_from_mut_ref<T>(reference: Option<&mut T>) -> *mut T{

    if let Some(address) = reference{
        address as *mut T
    }
    else{
        ptr::null_mut()
    }
}


/// Method like functions for C structs that cannot have a direct implementation


/// Creates a new sock_addr_in like a constructor
pub fn new_sock_addr_in(port: u16,ipv4: Ipv4Addr) -> SockAddrIn{

    SockAddrIn{
        sin_family: AF_INET as u16,
        sin_port: port.to_be(),
        sin_addr: in_addr{
            s_addr: u32::from(ipv4).to_be(),
        },
        sin_zero: [0;8],
    }

}


pub fn c_to_sock_addr(addr: &SockAddrIn) -> SocketAddrV4{

    // get the native byte order of the ip adddress
    let ip_octets = addr.sin_addr.s_addr.to_ne_bytes();
    // convert it to an ip address object
    let ip = Ipv4Addr::from(ip_octets);
    // get the port in current endianess from big endian
    let port = u16::from_be(addr.sin_port);

    SocketAddrV4::new(ip, port)
}

pub fn sock_addr_to_c(addr: &SocketAddrV4) -> SockAddrIn{
    // get the octets and port
    let ip_octets = addr.ip().octets();
    let port = addr.port();

    // the port will be in big endian
    SockAddrIn{
        sin_family: AF_INET as u16,
        sin_port: port.to_be(),
        sin_addr: in_addr{
            // get the 32 bits integer of the ip address
            s_addr: u32::from_ne_bytes(ip_octets),
        },
        sin_zero: [0;8],
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
