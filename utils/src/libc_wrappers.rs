use std::ffi::CString;
use std::fmt::{Debug};
use std::io::Error;
use libc::{__errno_location, recv, c_int, listen, c_char, c_void, sockaddr_in, AF_INET, setsockopt, accept, sockaddr, socklen_t, in_addr, size_t, getsockopt, close, dup, mode_t, shm_open, shm_unlink, sctp_initmsg, sockaddr_storage, sctp_sndrcvinfo};
use std::io::Result;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::{mem, ptr, slice};
use std::os::fd::RawFd;
use crate::sctp::sctp_api::SctpSenderReceiveInfo;

/// Common trait for all imported C structs
pub trait CStruct{
    fn new() -> Self where Self: Sized{
        unsafe{mem::zeroed()}
    }

    fn as_mut_bytes(&mut self) -> &mut [u8] where Self: Sized{
        unsafe{
            slice::from_raw_parts_mut(
                self as *mut Self as *mut u8,
                mem::size_of::<Self>()
            )
        }
    }

}

/// Aliases and structures that are not in libc
pub type SockAddrStorage = sockaddr_storage;


#[repr(C)]
#[derive(Copy, Clone,Default,Debug)]
pub struct InAddr {
    pub s_addr: u32,
}
impl CStruct for InAddr {}

#[repr(C)]
#[derive(Copy, Clone,Default,Debug)]
pub struct SockAddrIn {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: InAddr,
    pub sin_zero: [u8; 8],
}
impl CStruct for SockAddrIn {}

impl From<SocketAddrV4> for SockAddrIn {
    fn from(addr: SocketAddrV4) -> Self {

        let ip_octets = addr.ip().octets();
        let port = addr.port();

        let mut addr = InAddr::new();
        addr.s_addr = u32::from_ne_bytes(ip_octets);

        SockAddrIn{
            sin_family: AF_INET as u16,
            sin_port: port.to_be(),
            sin_addr: addr,
            sin_zero: [0;8],
        }
    }
}

impl From<SockAddrIn> for SocketAddrV4 {
    fn from(sock_addr: SockAddrIn) -> Self {
        let ip_bytes = sock_addr.sin_addr.s_addr.to_ne_bytes();
        let ip = Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);
        let port = u16::from_be(sock_addr.sin_port);

        SocketAddrV4::new(ip, port)
    }
}

impl SockAddrIn {
    pub fn from_ipv4(port: u16,ipv4: Ipv4Addr) -> Self{

        let ip_octets = ipv4.octets();
        let mut addr = InAddr::new();
        addr.s_addr = u32::from_ne_bytes(ip_octets);

        Self{
            sin_family: AF_INET as u16,
            sin_port: port.to_be(),
            sin_addr: addr,
            sin_zero: [0;8],
        }

    }

    pub fn as_c_counterpart(&mut self) -> *mut sockaddr_in{
        self as *mut Self as *mut sockaddr_in
    }
}



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
pub fn safe_accept(socket_fd: RawFd, address: Option<&mut SockAddrIn>, address_size: Option<&mut usize>) -> Result<RawFd>{

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


/// Debugging functions
pub fn debug_sockaddr(sockaddr: &SockAddrIn){
    println!("Sockaddr(family:{}, port:{}, address: {})",sockaddr.sin_family,sockaddr.sin_port.to_be(),Ipv4Addr::from(sockaddr.sin_addr.s_addr.to_be()));
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