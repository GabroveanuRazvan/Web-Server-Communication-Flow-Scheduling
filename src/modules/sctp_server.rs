extern crate libc;
use libc::{c_int, c_void, sockaddr_in, size_t, ssize_t, socklen_t, AF_INET, SOCK_SEQPACKET, IPPROTO_SCTP, INADDR_ANY, socket, bind, listen, recvmsg, sendmsg, SCTP_EOF, SCTP_ABORT};
use std::ptr;
use std::mem;
use std::net::Ipv4Addr;
use std::ffi::CStr;