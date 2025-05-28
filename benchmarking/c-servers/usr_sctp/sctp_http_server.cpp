#include <iostream>
#include <cerrno>
#include <cstring>
#include <string>
#include <sstream>

#include <unistd.h>
#include <sys/stat.h>
#include <netinet/in.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <usrsctp.h>

const uint16_t SCTP_PORT = 7878;
const uint16_t LOCAL_ENCAPSULATION_PORT = 11111;
const uint16_t REMOTE_ENCAPSULATION_PORT = 22222;

const int KILOBYTE = 1024;
const char* SERVER_ROOT = "./benchmark_raw_dataset";
const size_t CHUNK_SIZE = 16 * KILOBYTE;
const size_t SENDER_BUFFER_SIZE = 1024 * KILOBYTE;

const uint16_t MAX_STREAM_NUM = 4;


std::string recv_response_header(struct socket* sctp_socket,struct sockaddr_in& peer_addr) {

    std::string buffer;
    char header[8 * KILOBYTE];

    struct sctp_rcvinfo recv_info = {0};
    socklen_t info_len = sizeof(recv_info);
    unsigned int info_type;
    int flags;
    socklen_t peer_addr_size = sizeof(peer_addr);

    int bytes_received = usrsctp_recvv(sctp_socket,
                                       (void *) header,
                                      sizeof(header),
                                       (struct sockaddr*)&peer_addr,
                                      &peer_addr_size,
                                      (void *)&recv_info,
                                      &info_len,
                                      &info_type,
                                      &flags);

    // Return an empty buffer if the connection closed or an error occurred
    if(bytes_received == 0)
        return std::move(buffer);

    if (bytes_received < 0) {
        std::cerr << "Recv response: " << std::strerror(errno) << std::endl;
        return std::move(buffer);
    }

    buffer.append(header, bytes_received);

    return std::move(buffer);

}

// Extract the http path from a request and prepare it
std::string extract_http_path(const std::string& request) {
    std::istringstream request_stream(request);
    std::string method, path, version;

    request_stream >> method >> path >> version;

    if(method != "GET")
        std::cerr<<"Other verbs not supported"<<std::endl;

    return path == "/" ? "./index.html" : path.insert(0,".");

}

// Simple http header
std::string make_http_response_header(size_t size) {
    std::ostringstream response;

    response << "HTTP/1.1 200 OK\r\n"
             << "Content-Length: " << size << "\r\n"
             << "Content-Type: text/html\r\n"
             << "Connection: Keep-Alive\r\n"
             << "\r\n";

    return response.str();
}

bool send_file(struct socket* sctp_sock, const std::string& file_path,struct sockaddr_in& peer_addr){

    // Open the file
    int fd = open(file_path.c_str(), O_RDONLY);
    if (fd < 0){
        std::cerr << "Open: " << std::strerror(errno) << std::endl;
        usrsctp_close(sctp_sock);
        close(fd);
        return false;
    }

    // Get the file size
    struct stat file_status;
    if(fstat(fd,&file_status) < 0){
        std::cerr << "Fstat: " << std::strerror(errno) << std::endl;
        usrsctp_close(sctp_sock);
        close(fd);
        return false;
    }

    size_t file_size = file_status.st_size;

    // Map the file into memory
    char* mmap_file = (char *)mmap(nullptr,file_size,PROT_READ,MAP_PRIVATE,fd,0);
    if(mmap_file == nullptr){
        std::cerr << "Mmap: " << std::strerror(errno) << std::endl;
        usrsctp_close(sctp_sock);
        close(fd);
        return false;
    }

    uint16_t stream_index = 0;

    // Prepare a response header and send it
    auto response_header = make_http_response_header(file_size);

    // Send the file response
    struct sctp_sndinfo send_info;
    memset(&send_info, 0, sizeof(struct sctp_sndinfo));
    send_info.snd_sid = stream_index;
    send_info.snd_ppid = htonl(42);


    ssize_t bytes_sent = usrsctp_sendv(sctp_sock,
                                       response_header.c_str(),
                                       response_header.size(),
                                       nullptr,
                                       0,
                                       &send_info,
                                       sizeof(send_info),
                                       SCTP_SENDV_SNDINFO,
                                       0);

    stream_index = (stream_index + 1) % MAX_STREAM_NUM;

    if(bytes_sent < 0){
        std::cerr << "Sctp Send Response: " << std::strerror(errno) << std::endl;
        munmap(mmap_file,file_size);
        usrsctp_close(sctp_sock);
        close(fd);
        return false;
    }

    // Send the file in chunks until it is processed
    ssize_t current_sent = 0;
    while(current_sent < file_size){

        ssize_t bytes_to_send = std::min(CHUNK_SIZE,file_size-current_sent);

        send_info.snd_sid = stream_index;

        ssize_t bytes_sent = usrsctp_sendv(sctp_sock,
                                          mmap_file + current_sent,
                                          bytes_to_send,
                                           nullptr,
                                           0,
                                          &send_info,
                                          sizeof(send_info),
                                          SCTP_SENDV_SNDINFO,
                                          0);

        if(bytes_sent < 0){
            std::cerr << "Send: " << std::strerror(errno) << std::endl;
            munmap(mmap_file,file_size);
            usrsctp_close(sctp_sock);
            close(fd);
            return false;
        }

        current_sent += CHUNK_SIZE;

        stream_index = (stream_index + 1) % MAX_STREAM_NUM;

    }

    close(fd);
    munmap(mmap_file, file_size);
    return true;

}


int main(){

    // Change the working dir to the server root
    if (chdir(SERVER_ROOT) != 0) {
        std::cerr << "Chdir: " << strerror(errno) << std::endl;
        exit(EXIT_FAILURE);
    }

    // Sctp setup
    usrsctp_init(LOCAL_ENCAPSULATION_PORT, nullptr, nullptr);

    // Create the sctp socket
    struct socket* sctp_sock = usrsctp_socket(AF_INET,SOCK_STREAM,IPPROTO_SCTP,NULL,NULL,0,NULL);
    if(sctp_sock == nullptr){
        std::cerr << "Socket: " << std::strerror(errno) << std::endl;
        exit(EXIT_FAILURE);
    }

    // Enable usage of sctp context
    struct sctp_assoc_value av;
    av.assoc_id = SCTP_ALL_ASSOC;
    av.assoc_value = 47;

    if (usrsctp_setsockopt(sctp_sock, IPPROTO_SCTP, SCTP_CONTEXT, (const void*)&av, (socklen_t)sizeof(struct sctp_assoc_value)) < 0) {
        std::cerr << "usrsctp_setsockopt SCTP_CONTEXT: " << std::strerror(errno) << std::endl;
        exit(EXIT_FAILURE);
    }

    // Enable receive ancillary data
    const int on = 1;
    if (usrsctp_setsockopt(sctp_sock, IPPROTO_SCTP, SCTP_RECVRCVINFO, &on, sizeof(int)) < 0) {
        std::cerr << "usrsctp_setsockopt SCTP_RECVRCVINFO " << std::strerror(errno) << std::endl;
        exit(EXIT_FAILURE);
    }

    // Remote encapsulation port setup
    struct sctp_udpencaps encaps;
    memset(&encaps, 0, sizeof(struct sctp_udpencaps));
    encaps.sue_address.ss_family = AF_INET;
    encaps.sue_port = htons(REMOTE_ENCAPSULATION_PORT);
    if (usrsctp_setsockopt(sctp_sock, IPPROTO_SCTP, SCTP_REMOTE_UDP_ENCAPS_PORT, (const void*)&encaps, (socklen_t)sizeof(struct sctp_udpencaps)) < 0) {
        std::cerr << "usrsctp_setsockopt SCTP_REMOTE_UDP_ENCAPS_PORT" << std::strerror(errno) << std::endl;
        exit(EXIT_FAILURE);
    }

    sockaddr_in server_addr = {0};
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(SCTP_PORT);
    server_addr.sin_addr.s_addr = INADDR_ANY;

    // Bind the server to its address
    if(usrsctp_bind(sctp_sock, (sockaddr* ) &server_addr,sizeof(server_addr)) < 0){
        std::cerr << "Bind: " << std::strerror(errno) << std::endl;
        usrsctp_close(sctp_sock);
        exit(EXIT_FAILURE);
    }

    // Set the socket to listening mode
    if(usrsctp_listen(sctp_sock,1)  < 0){
        std::cerr << "Listen: " << std::strerror(errno) << std::endl;
        usrsctp_close(sctp_sock);
        exit(EXIT_FAILURE);
    }

    std::cout<<"Listening"<<std::endl;

    struct sockaddr_in peer_addr = {0};

    while(true){

        socklen_t len = 0;
        struct socket* client_sock = usrsctp_accept(sctp_sock, nullptr,&len);

        if(client_sock == nullptr){
            std::cerr << "Accept: " << std::strerror(errno) << std::endl;
            usrsctp_close(sctp_sock);
            exit(EXIT_FAILURE);
        }

        while (true){
            auto request = recv_response_header(client_sock,peer_addr);

            if(request.empty())
                break;

            auto path = extract_http_path(request);

            if(!send_file(client_sock,path,peer_addr))
                break;
        }


    }

    usrsctp_close(sctp_sock);
    return 0;
}

//export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH

//Total test time: 1070.48 secs
//Average throughput: 1.04578e+07 bytes/sec

//Total test time: 1022.55 secs
//Average throughput: 1.0948e+07 bytes/sec
