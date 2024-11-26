use std::io;
use std::thread::sleep;
use utils::cache::lru_cache::TempFileCache;
use utils::constants::{BYTE, KILOBYTE};
use utils::sctp::sctp_client::SctpPacketData;

fn main(){

    let mut buffer = [0;1 * KILOBYTE];

    let a = SctpPacketData::new(&buffer,0,0,0,0);

    for chunk in a.buffer.chunks(128){

        let b = SctpPacketData::new(&chunk,0,0,0,0);
        println!("{b:?}")

    }

    read1(&mut buffer);

    println!("{buffer:?}")


}

fn read1(buf: &mut [u8]){
    buf[0] = 1;
    read2(buf)
}

fn read2(buf: &mut [u8]){
    buf[1] = 2;
}
