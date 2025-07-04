#include <iostream>
#include <cerrno>
#include <cstring>
#include <string>
#include <sstream>

#include <unistd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <netinet/in.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <netinet/sctp.h>

const uint16_t PORT = 7878;
const int KILOBYTE = 1024;
const char* SERVER_ROOT = "../benchmark_raw_dataset";
const size_t CHUNK_SIZE = 16 * KILOBYTE;
const size_t SENDER_BUFFER_SIZE = 1024 * KILOBYTE;

uint16_t MAX_STREAM_NUM = 4;


std::string recv_response_header(int client_fd) {

    std::string buffer;
    char header[8 * KILOBYTE];

    struct sctp_sndrcvinfo recv_info = {0};
    int bytes_received = sctp_recvmsg(client_fd, header, sizeof(header), nullptr, nullptr,&recv_info, nullptr);

    // Return an empty buffer if the connection closed or an error occurred
    if(bytes_received == 0)
        return std::move(buffer);

    if (bytes_received < 0) {
        std::cerr << "Recv: " << std::strerror(errno) << std::endl;
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

bool send_file(int client_fd, const std::string& file_path){

    // Open the file
    int fd = open(file_path.c_str(), O_RDONLY);
    if (fd < 0){
        std::cerr << "Open: " << std::strerror(errno) << std::endl;
        close(client_fd);
        close(fd);
        return false;
    }

    // Get the file size
    struct stat file_status;
    if(fstat(fd,&file_status) < 0){
        std::cerr << "Fstat: " << std::strerror(errno) << std::endl;
        close(client_fd);
        close(fd);
        return false;
    }

    size_t file_size = file_status.st_size;

    // Map the file into memory
    char* mmap_file = (char *)mmap(nullptr,file_size,PROT_READ,MAP_PRIVATE,fd,0);
    if(mmap_file == nullptr){
        std::cerr << "Mmap: " << std::strerror(errno) << std::endl;
        close(client_fd);
        close(fd);
        return false;
    }

    uint16_t stream_index = 0;

    // Prepare a response header and send it
    auto response_header = make_http_response_header(file_size);

    if(sctp_sendmsg(client_fd,response_header.c_str(),response_header.size(), nullptr, 0,0,0,stream_index,.0,0) < 0){
        std::cerr << "Sctp Send: " << std::strerror(errno) << std::endl;
        munmap(mmap_file,file_size);
        close(client_fd);
        close(fd);
        return false;
    }

    stream_index = (stream_index + 1) % MAX_STREAM_NUM;

    // Send the file in chunks until it is processed
    size_t current_sent = 0;
    while(current_sent < file_size){

        size_t bytes_to_send = std::min(CHUNK_SIZE,file_size-current_sent);

        size_t bytes_sent = sctp_sendmsg(client_fd,
                                         mmap_file + current_sent,
                                         bytes_to_send,
                                         nullptr,
                                         0,
                                         0,
                                         0,
                                         stream_index,
                                         0,
                                         0);
        if(bytes_sent < 0){
            std::cerr << "Send: " << std::strerror(errno) << std::endl;
            munmap(mmap_file,file_size);
            close(client_fd);
            close(fd);
            return false;
        }

        stream_index = (stream_index + 1) % MAX_STREAM_NUM;
        current_sent += CHUNK_SIZE;

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

    // Create the sctp socket
    int sock_fd = socket(AF_INET,SOCK_STREAM,IPPROTO_SCTP);
    if(sock_fd < 0){
        std::cerr << "Socket: " << std::strerror(errno) << std::endl;
        exit(EXIT_FAILURE);
    }

    sockaddr_in server_addr = {0};
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(PORT);
    server_addr.sin_addr.s_addr = INADDR_ANY;

    // Reuse the address when needed
    int opt = 1;
    if(setsockopt(sock_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt)) < 0){
        std::cerr << "setsockopt: " << std::strerror(errno) << std::endl;
        close(sock_fd);
        exit(EXIT_FAILURE);
    }


    // Enable data io
    struct sctp_event_subscribe events;
    bzero(&events,sizeof(events));
    events.sctp_data_io_event = 1;

    if(setsockopt(sock_fd,IPPROTO_SCTP,SCTP_EVENTS,&events,sizeof(events)) < 0){
        perror("setsockopt");
        close(sock_fd);
        exit(EXIT_FAILURE);
    }

    // Bind the server to its address
    if(bind(sock_fd, (sockaddr* ) &server_addr,sizeof(server_addr)) < 0){
        std::cerr << "Bind: " << std::strerror(errno) << std::endl;
        close(sock_fd);
        exit(EXIT_FAILURE);
    }

    // Set the socket to listening mode
    if(listen(sock_fd,1)  < 0){
        std::cerr << "Listen: " << std::strerror(errno) << std::endl;
        close(sock_fd);
        exit(EXIT_FAILURE);
    }

    while(true){

        sockaddr_in client_addr = {0};
        socklen_t client_addr_size = sizeof(client_addr);

        // Accept a new client
        int client_fd = accept(sock_fd,(sockaddr*) &client_addr,&client_addr_size);

        if(client_fd < 0){
            std::cerr << "Accept: " << std::strerror(errno) << std::endl;
            continue;
        }

        // Set the client receive buffer to provided size
        if(setsockopt(client_fd, SOL_SOCKET, SO_SNDBUF, &SENDER_BUFFER_SIZE, sizeof(SENDER_BUFFER_SIZE)) < 0){
            std::cerr << "setsockopt client: " << std::strerror(errno) << std::endl;
            close(client_fd);
            continue;
        }

        std::cout<<"New connection"<< std::endl;

        int actual_size = 0;
        unsigned int len = sizeof(actual_size);

        // Get the actual size of the kernel receive buffer
        if(getsockopt(client_fd, SOL_SOCKET, SO_SNDBUF, &actual_size, &len) < 0){
            std::cerr << "getsockopt client: " << std::strerror(errno) << std::endl;
            close(client_fd);
            continue;
        }

        std::cout << "Size of kernel send buffer: "<< actual_size << std::endl;

        while(true){
            auto request = recv_response_header(client_fd);

            if(request.empty())
                break;

            auto path = extract_http_path(request);

            if(!send_file(client_fd,path))
                break;

        }

        close(client_fd);

    }

    close(sock_fd);
    return 0;
}