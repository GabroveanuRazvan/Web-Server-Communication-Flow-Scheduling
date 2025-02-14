use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::io::Error;
use libc::{__errno_location, recv, c_int, listen, c_char, c_void, sockaddr_in, AF_INET, sctp_sndrcvinfo, setsockopt, accept, sockaddr, socklen_t, in_addr, size_t, getsockopt, close, dup, mode_t, shm_open, shm_unlink};
use std::io::Result;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::{mem, ptr};
use std::os::fd::RawFd;

/// Aliases and structures that are not in libc

pub type SockAddrIn = sockaddr_in;
pub type SctpSenderInfo = sctp_sndrcvinfo;

/// FFI bindings for functions that the libc crate does not provide
extern "C"{
    fn inet_pton(af: c_int,src: *const c_char,dst: *mut c_void) -> c_int;
}

/// Wrapper for listen, returns Ok(0) or Err(io::Error) on failure
pub fn safe_listen(socket_fd: RawFd,max_queue_size: i32) -> Result<i32> {

    let result = unsafe{
        listen(socket_fd, max_queue_size)
    };

    wrap_result_nonnegative(result)

}

/// Wrapper for accept, returns Ok(0) or Err(io::Error) on failure
pub fn safe_accept(socket_fd: RawFd, address: Option<&mut SockAddrIn>, address_size: Option<&mut usize>) -> Result<i32>{


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
pub fn safe_setsockopt(socket_fd: RawFd, level:i32, option_name:i32, option_value:&[u8]) -> Result<i32>{

    let option_length = option_value.len() as u32;

    let result = unsafe{
        setsockopt(socket_fd,level,option_name,option_value.as_ptr() as *const c_void,option_length)
    };

    wrap_result_nonnegative(result)

}

/// Wrapper function used to get the socket options
pub fn safe_getsockopt(socket_fd: RawFd, level: i32, option_name: i32, option_value: &mut [u8]) -> Result<i32>{
    let mut option_length = option_value.len() as u32;

    let result = unsafe{
        getsockopt(socket_fd,level,option_name,option_value.as_ptr() as *mut c_void,&mut option_length as *mut u32)
    };

    wrap_result_nonnegative(result)
}

/// Wrapper function used for recv
pub fn safe_recv(socket_fd: RawFd, msg: &mut [u8],message_size: usize,flags: i32) -> Result<i32>{

    let result = unsafe{
        recv(socket_fd,msg.as_mut_ptr() as *mut c_void,message_size as size_t,flags) as i32
    };

    wrap_result_nonnegative(result)
}

/// Wrapper function for close
pub fn safe_close(socket_fd: RawFd) -> Result<i32>{

    let result = unsafe{
        close(socket_fd)
    };

    wrap_result_nonnegative(result)
}

/// Wrapper function for dup.
pub fn safe_dup(old_fd: RawFd) -> Result<RawFd>{
    let result = unsafe{
        dup(old_fd)
    };

    wrap_result_nonnegative(result)
}

/// Wrapper function for shm_open
pub fn safe_shm_open(name: &str,oflag: i32,mode: u32) -> Result<RawFd>{

    // convert the string into a string one (null terminated)
    let name = CString::new(name)?;

    let result = unsafe{
        shm_open(name.as_ptr() as *const c_char,oflag,mode)
    };

    wrap_result_nonnegative(result)

}

/// Wrapper function for shm_unlink
pub fn safe_shm_unlink(name: &str) -> Result<i32>{

    // convert the string into a string one (null terminated)
    let name = CString::new(name)?;

    let result = unsafe{
        shm_unlink(name.as_ptr() as *const c_char)
    };

    wrap_result_nonnegative(result)
}


/// Function that extracts errno safely.
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

/// Creates an empty sctp_sndrinfo
pub fn new_sctp_sndrinfo() -> SctpSenderInfo{

    let info: SctpSenderInfo = unsafe { mem::zeroed() };
    info
}


/// Function that transforms a C sock_addr_in into a rust SocketAddrV4
pub fn c_to_sock_addr(addr: &SockAddrIn) -> SocketAddrV4{

    // get the native byte order of the ip adddress
    let ip_octets = addr.sin_addr.s_addr.to_ne_bytes();
    // convert it to an ip address object
    let ip = Ipv4Addr::from(ip_octets);
    // get the port in current endianess from big endian
    let port = u16::from_be(addr.sin_port);

    SocketAddrV4::new(ip, port)
}

/// Function that transforms a rust SocketAddrV4 into a C sock_addr_in
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
    println!("\nSCTP Send/Receive Info:");
    println!("  Stream: {}", info.sinfo_stream);
    println!("  SSN: {}", info.sinfo_ssn);
    println!("  Flags: {}", info.sinfo_flags);
    println!("  PPID: {}", info.sinfo_ppid);
    println!("  Context: {}", info.sinfo_context);
    println!("  TSN: {}", info.sinfo_tsn);
    println!("  Cumulative TSN: {}", info.sinfo_cumtsn);
    println!("  Association ID: {}\n", info.sinfo_assoc_id);
}


/// Builders


/// File mode builder for the C mode_t: https://man7.org/linux/man-pages/man3/mode_t.3type.html#top_of_page
pub struct ModeBuilder{
    mode: u32,
}

impl ModeBuilder {
    pub fn new() -> Self {
        Self { mode: 0 }
    }

    pub fn user_read(mut self) -> Self {
        self.mode |= libc::S_IRUSR;
        self
    }

    pub fn user_write(mut self) -> Self {
        self.mode |= libc::S_IWUSR;
        self
    }

    pub fn user_execute(mut self) -> Self {
        self.mode |= libc::S_IXUSR;
        self
    }

    pub fn group_read(mut self) -> Self {
        self.mode |= libc::S_IRGRP;
        self
    }

    pub fn group_write(mut self) -> Self {
        self.mode |= libc::S_IWGRP;
        self
    }

    pub fn group_execute(mut self) -> Self {
        self.mode |= libc::S_IXGRP;
        self
    }

    pub fn others_read(mut self) -> Self {
        self.mode |= libc::S_IROTH;
        self
    }

    pub fn others_write(mut self) -> Self {
        self.mode |= libc::S_IWOTH;
        self
    }

    pub fn others_execute(mut self) -> Self {
        self.mode |= libc::S_IXOTH;
        self
    }

    pub fn build(self) -> libc::mode_t {
        self.mode
    }
}