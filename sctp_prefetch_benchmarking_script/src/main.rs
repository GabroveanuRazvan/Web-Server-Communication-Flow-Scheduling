use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{SocketAddrV4};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;
use utils::config::serialization::load;
use utils::constants::{KILOBYTE};
use utils::libc_wrappers::CStruct;
use utils::sctp::sctp_api::{SctpEventSubscribe, SctpPeerBuilder, SctpSenderReceiveInfo};
use utils::sctp::sctp_client::{SctpStreamBuilder};

const PEER_ADDRESS: &str = "192.168.50.30:7878";
const RECEIVE_BUFFER_SIZE: usize = 16 * KILOBYTE;
const USER_COUNT: usize = 4;
const REQUESTS_PATH_PREFIX: &str = "../benchmarking/requests/prefetch_requests_";
const REQUESTS_PATH_SUFFIX: &str = "_5000.json";
const RUNS_COUNT: usize = 1;


fn main() {

    let mut results = Vec::with_capacity(RUNS_COUNT);

    (0..RUNS_COUNT).into_iter().for_each(|idx| {

        let requests_path = format!("{REQUESTS_PATH_PREFIX}{idx}{REQUESTS_PATH_SUFFIX}");
        
        let requests: VecDeque<String> = load(requests_path).unwrap();
        let requests = Arc::new(Mutex::new(requests));

        let mut threads = Vec::with_capacity(USER_COUNT);

        let socket_address: SocketAddrV4 = PEER_ADDRESS.parse().unwrap();

        let mut events = SctpEventSubscribe::new();
        events.sctp_data_io_event = 1;

        let mut user = SctpStreamBuilder::new()
            .socket()
            .address(socket_address.ip().clone())
            .port(socket_address.port())
            .set_incoming_streams(USER_COUNT as u16)
            .set_outgoing_streams(USER_COUNT as u16)
            .events(events)
            .ttl(0)
            .build();

        user.events();
        user.connect();

        let user = Arc::new(user);

        // Create the channels to communicate with the users
        let mut worker_receivers = Vec::with_capacity(USER_COUNT);
        let mut worker_senders = Vec::with_capacity(USER_COUNT);

        (0..USER_COUNT).for_each(|_| {
            let (tx, rx) = mpsc::channel();
            worker_senders.push(tx);
            worker_receivers.push(rx);
        });

        let receiver_user = Arc::clone(&user);
        let receiver_thread = thread::spawn(move || {
            let mut buffer = vec![0u8; RECEIVE_BUFFER_SIZE];

            loop {
                let mut sender_info = SctpSenderReceiveInfo::new();

                match receiver_user.read(&mut buffer, Some(&mut sender_info), None) {
                    Ok(0) => break,

                    Err(err) => {
                        eprintln!("{}", err);
                        break;
                    }

                    Ok(n) => {
                        let stream_index = sender_info.sinfo_stream as usize;
                        worker_senders[stream_index].send((buffer[0..n].to_vec(), sender_info.sinfo_ppid)).unwrap();
                    }
                }
            }
        });

        worker_receivers.into_iter().enumerate().for_each(|(idx, receiver)| {
            let user = Arc::clone(&user);
            let stream_number = idx as u16;

            let mut total_size = 0;
            let mut total_time = 0.0;
            let requests = Arc::clone(&requests);

            threads.push(
                thread::spawn(move || {
                    loop {
                        let request = {
                            let mut guard = requests.lock().unwrap();

                            match guard.pop_back() {
                                Some(request) => request,
                                None => break,
                            }
                        };

                        let start = Instant::now();
                        user.write_all(request.as_bytes(), stream_number, 0, 0).unwrap();

                        let (buffer, ppid) = receiver.recv().unwrap();

                        let file_size = usize::from_be_bytes(buffer[0..8].try_into().unwrap());

                        let mut current_length = 0;

                        while current_length < file_size {
                            let (buffer, ppid) = receiver.recv().unwrap();
                            let bytes_received = buffer.len();
                            current_length += bytes_received;
                        }

                        let end = start.elapsed().as_secs_f64();

                        total_time += end;
                        total_size += file_size;
                    }

                    let throughput = total_size as f64 / total_time;

                    (total_time, total_size, throughput)
                })
            );
        });


        let mut avg_throughput = 0.0;
        let mut avg_time = 0.0;


        threads.into_iter().for_each(|thread| {
            let (time, size, throughput) = thread.join().unwrap();
            avg_throughput += throughput;
            avg_time += time;
        });

        avg_throughput /= USER_COUNT as f64;
        avg_time /= USER_COUNT as f64;

        println!("Avg time: {avg_time}");
        println!("Avg throughput: {avg_throughput}");

        results.push((avg_time,avg_throughput));
    });
    
    println!("{:#?}",results);
}


fn extract_content_length(buffer: &[u8]) -> Option<usize>{

    let text = String::from_utf8_lossy(buffer);

    for line in text.lines(){

        if let Some(rest) = line.to_ascii_lowercase().strip_prefix("content-length: ") {
            return rest.trim().parse::<usize>().ok();
        }

    }

    None

}


// Total test time:        1399.9743759190005
// Average throughput:     8539839.710247297
//
//
// Total test time:        1618.7891264680002
// Average throughput:     7389004.508294408



// TCP non-persistent
// Avg time: 534.8702142852501
// Avg throughput: 2592908.7019724506

// TCP persistent
// Avg time: 548.1597555410002
// Avg throughput: 2530119.323492427

// Sctp
// Avg time: 674.6722919482498
// Avg throughput: 2055641.967823537


// 10k

// TCP persistent
// Avg time: 1096.6419685530009
// Avg throughput: 2558931.6497239224

// Sctp
// Avg time: 1401.3927615809998
// Avg throughput: 2002461.489028103





// Tcp persistent

// [
// (
// 249.78680205175024,
// 5552288.981161752,
// ),
// (
// 236.50500676674977,
// 5564475.525613305,
// ),
// (
// 251.9222534002498,
// 5547841.3885640325,
// ),
// (
// 249.5610571387499,
// 5515715.55129152,
// ),
// (
// 249.64131678449994,
// 5451687.080260607,
// ),
// (
// 251.62677841700017,
// 5434580.913087684,
// ),
// ]


// TCP non persistent
// [
// (
// 271.14068072724984,
// 5114999.349618869,
// ),
// (
// 266.25268239499974,
// 4942831.704526664,
// ),
// (
// 277.7004658617499,
// 5032846.49342877,
// ),
// (
// 280.43517022550003,
// 4908346.063409907,
// ),
// (
// 275.8002831325,
// 4934582.493498794,
// ),
// (
// 278.19011157949984,
// 4915701.38108216,
// ),
// ]

// SCTP

// Avg time: 491.30781805674997
// Avg throughput: 2822847.3393492326
