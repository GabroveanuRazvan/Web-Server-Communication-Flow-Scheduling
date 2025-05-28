#include <iostream>
#include <cerrno>
#include <cstring>
#include <string>
#include <sstream>
#include <vector>
#include <fstream>
#include <chrono>


#include <unistd.h>
#include <usrsctp.h>
#include <sys/stat.h>
#include <netinet/in.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <arpa/inet.h>


const size_t KILOBYTE = 1024;

const char* REQUESTS_FILE_PATH = "./requests.txt";
const char* PEER_IPV4 = "192.168.50.30";

const uint16_t LOCAL_ENCAPSULATION_PORT = 22222;
const uint16_t REMOTE_ENCAPSULATION_PORT = 11111;
const uint16_t SCTP_SERVER_PORT = 7878;

const size_t RECEIVE_BUFFER_SIZE = 1024 * KILOBYTE;


std::vector<std::string> load_requests(const std::string& filename) {
    std::vector<std::string> requests;
    std::ifstream infile(filename);
    std::string request;

    if (!infile) {
        std::cerr << "Could not open file: " << filename << std::endl;
        return requests;
    }

    while (std::getline(infile, request)) {
        requests.push_back(request);
    }

    return requests;
}


std::string HttpGetHeader(const std::string& file_path) {

    if (file_path.empty()) {
        throw std::runtime_error("Empty Http Header request");
    }

    return "GET " + file_path + " HTTP/1.1\r\nHost: cpp";
}


unsigned long long extract_content_length(const char* buffer) {
    const char* header = "Content-Length: ";
    const char* found = std::strstr(buffer, header);

    if (!found) return -1;

    found += std::strlen(header);

    return std::strtoull(found, nullptr, 10);;
}


int main(){

    char buffer[RECEIVE_BUFFER_SIZE];
    std::vector<std::string> requests = load_requests(REQUESTS_FILE_PATH);
    size_t request_count = requests.size();

    double total_run_time = 0;
    size_t total_received_size = 0;

    // Sctp init
    usrsctp_init(LOCAL_ENCAPSULATION_PORT, NULL, NULL);

    // Create the sctp socket
    struct socket* sctp_sock = usrsctp_socket(AF_INET, SOCK_STREAM, IPPROTO_SCTP, nullptr, nullptr, 0, nullptr);
    if (sctp_sock == nullptr) {
        std::cerr << "socket: " << std::strerror(errno) << std::endl;
        exit(EXIT_FAILURE);
    }

    // Bind the socket to an ephemeral port
    sockaddr_in addr4 = {0};
    addr4.sin_family = AF_INET;
    addr4.sin_port = htons(0);
    addr4.sin_addr.s_addr = INADDR_ANY;
    if (usrsctp_bind(sctp_sock, (struct sockaddr *)&addr4, sizeof(struct sockaddr_in)) < 0) {
        std::cerr << "bind: " << std::strerror(errno) << std::endl;
        usrsctp_close(sctp_sock);
        exit(EXIT_FAILURE);
    }

    // Set the remote port
    struct sctp_udpencaps encaps;
    memset(&encaps, 0, sizeof(struct sctp_udpencaps));
    encaps.sue_address.ss_family = AF_INET;
    encaps.sue_port = htons(REMOTE_ENCAPSULATION_PORT);
    if (usrsctp_setsockopt(sctp_sock, IPPROTO_SCTP, SCTP_REMOTE_UDP_ENCAPS_PORT, (const void*)&encaps, (socklen_t)sizeof(struct sctp_udpencaps)) < 0) {
        std::cerr << "setsockopt: " << std::strerror(errno) << std::endl;
        usrsctp_close(sctp_sock);
        exit(EXIT_FAILURE);
    }

    // Enable receive ancillary data
    const int on = 1;
    if (usrsctp_setsockopt(sctp_sock, IPPROTO_SCTP, SCTP_RECVRCVINFO, &on, sizeof(int)) < 0) {
        std::cerr << "usrsctp_setsockopt SCTP_RECVRCVINFO " << std::strerror(errno) << std::endl;
        exit(EXIT_FAILURE);
    }

    // Build the peer socket address
    sockaddr_in peer_addr = {0};
    socklen_t addr_size = sizeof(peer_addr);

    peer_addr.sin_family = AF_INET;
    peer_addr.sin_port = htons(SCTP_SERVER_PORT);
    if(inet_pton(AF_INET, PEER_IPV4, &peer_addr.sin_addr) < 0){
        std::cerr << "inet_pton: "<< std::strerror(errno) << std::endl;
        usrsctp_close(sctp_sock);
        exit(EXIT_FAILURE);
    }

    std::cout<<"Connecting..."<<std::endl;

    // Connect to the peer
    if(usrsctp_connect(sctp_sock,(struct sockaddr*)&peer_addr,sizeof(peer_addr)) < 0){
        std::cerr << "connect: " << std::strerror(errno) << std::endl;
        usrsctp_close(sctp_sock);
        exit(EXIT_FAILURE);
    }

    std::cout<<"Connected!"<<std::endl;

    int request_index = 0;

    struct sctp_sndinfo send_info = {0};
    send_info.snd_sid = 0;
    send_info.snd_ppid = htonl(42);

    struct sctp_rcvinfo rcv_info = {0};
    socklen_t info_len = sizeof(rcv_info);

    for(const auto& request : requests){

        auto header = HttpGetHeader(request);

        ssize_t bytes_sent = usrsctp_sendv(sctp_sock,
                                         header.c_str(),
                                         header.size(),
                                         nullptr,
                                         0,
                                         &send_info,
                                         sizeof(send_info),
                                         SCTP_SENDV_SNDINFO,
                                         0);


        if(bytes_sent < 0){
            std::cerr << "Send msg: " << std::strerror(errno) << std::endl;
            usrsctp_close(sctp_sock);
            exit(EXIT_FAILURE);
        }

        unsigned int info_type;
        int flags;

        // Receive the response
        ssize_t bytes_received = usrsctp_recvv(sctp_sock,
                                               (void*)buffer,
                                               RECEIVE_BUFFER_SIZE,
                                               (struct sockaddr *) &peer_addr,
                                               &addr_size,
                                               (void *)&rcv_info,
                                               &info_len,
                                               &info_type,
                                               &flags);


        if(bytes_received < 0){
            std::cerr << "Recv msg: " << std::strerror(errno) << std::endl;
            usrsctp_close(sctp_sock);
            exit(EXIT_FAILURE);
        }

        // Connection closed
        if(bytes_received == 0){
            break;
        }

        // Start the timer when receiving the file
        auto start_time = std::chrono::high_resolution_clock::now();

        size_t content_length = extract_content_length(buffer);
        size_t current_length = 0;

        // Receive the data in a loop until the file is downloaded
        while(current_length < content_length){

            // Receive the response
            bytes_received = usrsctp_recvv(sctp_sock,
                                           (void*)buffer,
                                           RECEIVE_BUFFER_SIZE,
                                           (struct sockaddr *) &peer_addr,
                                           &addr_size,
                                           (void *)&rcv_info,
                                           &info_len,
                                           &info_type,
                                           &flags);



            if(bytes_received < 0){
                std::cerr << "Recv msg: " << std::strerror(errno) << std::endl;
                usrsctp_close(sctp_sock);
                exit(EXIT_FAILURE);
            }

            // Connection closed
            if(bytes_received == 0){
                break;
            }

            current_length += bytes_received;

        }

        // Stop the timer and compute the data
        auto end_time = std::chrono::high_resolution_clock::now();
        std::chrono::duration<double> duration = end_time - start_time;

        total_run_time += duration.count();
        total_received_size += content_length;


        std::cout<<++request_index<<std::endl;
    }

    double throughput = (double)total_received_size / total_run_time;

    std::cout<<"Total test time: "<<total_run_time<<" secs"<<std::endl;
    std::cout<<"Average throughput: "<<throughput<<" bytes/sec"<<std::endl;

    usrsctp_close(sctp_sock);
    return 0;
}