use std::fmt::format;
use std::thread;
use std::time::Duration;
use utils::packets::byte_packet::BytePacket;
use utils::tcp::tcp_association::{TcpAssociation, TcpAssociationListener};

fn main() {

   let stream_count = 12;
   
   let mut assoc = TcpAssociation::connect("192.168.50.30:7878",stream_count).unwrap();
   
   let stream_count = assoc.stream_count();
   println!("Stream count {}", stream_count);
   
   let mut current_stream = 0;
   
   for i in 0..50{
      let file_path = format!("/3.00M-6.00M/{}.html",i);

      assoc.send(file_path.as_bytes(),current_stream,1).unwrap();

      let metadata_message = assoc.receive().unwrap();
      let mut byte_packet = BytePacket::from(metadata_message.message.as_slice());
      let file_size = byte_packet.read_u64().unwrap() as usize;
      let file_path = String::from_utf8_lossy(byte_packet.read_all().unwrap());
      let mut message = String::new();
      let mut current_size = 0;
      let stream = metadata_message.stream;
      
      while current_size < file_size {

         let message_info = assoc.receive().unwrap();
         message.push_str(String::from_utf8_lossy(message_info.message.as_slice()).as_ref());
         current_size += message_info.message.len();

      }
      println!("Received! {:?} {} {stream}", file_path,file_size);
      current_stream += 1;
      current_stream = (current_stream + 1) % stream_count;
   }
   
   
   
}
