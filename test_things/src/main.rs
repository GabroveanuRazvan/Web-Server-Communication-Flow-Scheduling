use std::collections::{HashMap, HashSet};
use std::{fs, mem, slice, thread};
use std::cell::RefCell;
use memmap2::{Mmap, MmapMut};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, RwLock};
use path_clean::PathClean;
use utils::html_prefetch_service::HtmlPrefetchService;
use utils::http_parsers::extract_http_paths;
use std::num::Wrapping;
use std::time::Duration;
use libc::{listen, setsockopt, IPPROTO_SCTP, SCTP_INITMSG, SCTP_STATUS};
use utils::libc_wrappers::{safe_accept, safe_getsockopt, safe_listen, safe_setsockopt, CStruct, SockAddrIn};
use utils::pools::indexed_thread_pool::IndexedTreadPool;
use utils::sctp::sctp_api::{safe_sctp_bindx, safe_sctp_connectx, safe_sctp_socket, SctpEventSubscribe, SctpInitMsg, SctpPeerAddrInfo, SctpStatus, SCTP_BINDX_ADD_ADDR};

fn main(){

    let sockfd = safe_sctp_socket().unwrap();

    let sockaddr: SockAddrIn = SocketAddrV4::new(Ipv4Addr::new(127,0,0,1), 9898).into();

    let mut sctp_init: SctpInitMsg = SctpInitMsg::new();
    sctp_init.sinit_num_ostreams = 20;
    sctp_init.sinit_max_instreams = 9;

    safe_setsockopt(sockfd,IPPROTO_SCTP,SCTP_INITMSG,sctp_init.as_mut_bytes()).unwrap();


     safe_sctp_connectx(sockfd,&mut [sockaddr]).unwrap();



    let mut status: SctpStatus = SctpStatus::new();
    safe_getsockopt(sockfd,IPPROTO_SCTP,SCTP_STATUS,status.as_mut_bytes()).unwrap();

    println!("{} {}",status.sstat_instrms,status.sstat_outstrms);




    println!("{}",mem::size_of::<SctpInitMsg>());

}
