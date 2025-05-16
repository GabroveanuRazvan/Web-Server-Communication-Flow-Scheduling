use std::thread;
use std::time::Duration;
use utils::tcp::tcp_association::{TcpAssociation, TcpAssociationListener};

fn main() {


    let th2 = thread::spawn(|| {
        let listener = TcpAssociationListener::bind("0.0.0.0:7878",5).unwrap();
        thread::sleep(Duration::from_millis(10));
        let (mut assoc,addresses) =  listener.accept().unwrap();
        println!("{:?}",addresses);
        assoc.send(b"1234",2,10).unwrap();
        
    });
    
    
    
    let th1 = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
        let assoc = TcpAssociation::connect("127.0.0.1:7878",5).unwrap();
        let mut assoc = assoc.try_clone().unwrap();
        let message = assoc.receive().unwrap();
        println!("{:?}",message);
        println!("{:?}",String::from_utf8_lossy(&message.message));
        
    });
    
    
    
    th1.join().unwrap();
    th2.join().unwrap();
    
}
