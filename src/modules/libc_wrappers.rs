use std::ffi::CString;
use std::fmt;
use std::io::Error;
use libc::{__errno_location, c_int, listen, c_char, c_void, sockaddr_in, AF_INET};
use std::io::Result;

/// used for naming conventions

pub type SockAddrIn = sockaddr_in;

extern "C"{
    fn inet_pton(af: c_int,src: *const c_char,dst: *mut c_void) -> c_int;
}

/// wrapper for listen
pub fn safe_listen(socket_fd: i32,max_queue_size: i32) -> Result<i32> {

    let result = unsafe{
        listen(socket_fd, max_queue_size)
    };

    wrap_result_nonnegative(result)

}

/// wrapper for AF_INET inet_pton
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

/// wrapper function for nonnegative values
pub fn wrap_result_nonnegative(result: i32) -> Result<i32> {

    if result >= 0{
        Ok(result)
    }
    else{
        Err(Error::from_raw_os_error(get_errno()))
    }

}

/// wrapper function for positive values
pub fn wrap_result_positive(result: i32) -> Result<i32> {

    if result > 0{
        Ok(result)
    }
    else{
        Err(Error::from_raw_os_error(get_errno()))
    }

}

/// Debugging function
pub fn debug_sockaddr(sockaddr: &SockAddrIn){
    println!("Sockaddr(family:{}, port:{}, address: {})",sockaddr.sin_family,sockaddr.sin_port.to_be(),sockaddr.sin_addr.s_addr);
}