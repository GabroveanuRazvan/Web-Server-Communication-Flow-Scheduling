use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::{Path};
use std::io::Result;
use crate::config::tcp_assoc_server_config::TcpAssocServerConfig;
use crate::pools::scheduling::scheduling_policy::SchedulingPolicy;
use crate::pools::scheduling::tcp_connection_scheduler::TcpConnectionScheduler;
use crate::tcp::tcp_association::{TcpAssociation, TcpAssociationListener};

pub struct TcpAssocServer{
    assoc_listener: TcpAssociationListener
}

impl TcpAssocServer{
    
    pub fn start(mut self) -> Result<()>{
        
        println!("Server started and listening on {:?}",self.assoc_listener.local_addr());
        println!("Current directory: {}",env::current_dir().unwrap().display());

        for assoc in self.assoc_listener.incoming(){

            let assoc = assoc?;
            Self::handle_client(assoc)?

        }

        Ok(())
        
    }
    
    pub fn handle_client(assoc: TcpAssociation) -> Result<()>{
        
        println!("Connected to {:#?}",assoc.peer_addresses());
        println!("Scheduling policy: {:?}",TcpAssocServerConfig::scheduling_policy());
        
        match TcpAssocServerConfig::scheduling_policy(){
            SchedulingPolicy::ShortestConnectionFirst =>{
                
                let scheduler = TcpConnectionScheduler::new(assoc.stream_count() as usize,
                                                            assoc,
                                                            TcpAssocServerConfig::file_packet_size());
                
                scheduler.start();
                
            },
            
            _ => panic!("Unknown scheduling policy"), 
        }
        
        Ok(())
    }
    
}

pub struct TcpAssocServerBuilder{
    stream_count: u8,
    address: SocketAddrV4,
}

impl TcpAssocServerBuilder{
    
    pub fn new() -> Self{
        Self{
            stream_count: 0,
            address: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED,0),
        }
    }
    
    pub fn server_root(self,path: impl AsRef<Path>) -> Self{
        
        env::set_current_dir(path).expect("Failed to set server root");
        self
        
    }
    
    pub fn ipv4(mut self, address: Ipv4Addr) -> Self{
        self.address.set_ip(address);
        self
    }
    
    pub fn port(mut self, port: u16) -> Self{
        self.address.set_port(port);
        self
    }
    
    pub fn stream_count(mut self,stream_count: u8) -> Self{
        self.stream_count = stream_count;
        self
    }
    
    pub fn build(self) -> TcpAssocServer{
        
        TcpAssocServer{
            assoc_listener: TcpAssociationListener::bind(self.address,self.stream_count).expect("Failed to bind to address"),
        }
        
    }
    
    
}