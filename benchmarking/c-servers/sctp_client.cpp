#include <iostream>
#include <cerrno>
#include <cstring>
#include <string>
#include <sstream>
#include <vector>
#include <fstream>
#include <chrono>


#include <unistd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <netinet/in.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <arpa/inet.h>
#include <netinet/sctp.h>

const size_t KILOBYTE = 1024;

const char* REQUESTS_FILE_PATH = "./requests.txt";
const char* PEER_IPV4 = "127.0.0.1";
const u_int16_t PORT = 7878;
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

    // Create the sctp socket
    int sock_fd = socket(AF_INET,SOCK_STREAM,IPPROTO_SCTP);
    if(sock_fd < 0){
        std::cerr << "socket: " << std::strerror(errno) << std::endl;
        exit(EXIT_FAILURE);
    }

    // Build the peer socket address
    sockaddr_in peer_addr = {0};
    peer_addr.sin_family = AF_INET;
    peer_addr.sin_port = htons(PORT);
    if(inet_pton(AF_INET, PEER_IPV4, &peer_addr.sin_addr) < 0){
        std::cerr << "inet_pton: "<< std::strerror(errno) << std::endl;
        close(sock_fd);
        exit(EXIT_FAILURE);
    }

    // Reuse the address when needed
    int option = 1;
    if(setsockopt(sock_fd, SOL_SOCKET, SO_REUSEADDR, &option, sizeof(option)) < 0){
        std::cerr << "setsockopt: " << std::strerror(errno) << std::endl;
        close(sock_fd);
        exit(EXIT_FAILURE);
    }

    // Connect to the peer
    if(connect(sock_fd,(struct sockaddr*)&peer_addr,sizeof(peer_addr)) == -1){
        std::cerr << "connect: " << std::strerror(errno) << std::endl;
        close(sock_fd);
        exit(EXIT_FAILURE);
    }

    int request_index = 0;
    for(const auto& request : requests){

        auto header = HttpGetHeader(request);



        // Send the request
        if(sctp_sendmsg(sock_fd,header.c_str(),header.size(), nullptr,0,0,0,0,0,0) < 0){
            std::cerr << "Send msg: " << std::strerror(errno) << std::endl;
            close(sock_fd);
            exit(EXIT_FAILURE);
        }

        // Receive the response
        int bytes_received = 0;

        if(( bytes_received = sctp_recvmsg(sock_fd,buffer,sizeof(buffer), nullptr, nullptr, nullptr, nullptr)) < 0){
            std::cerr << "Recv msg: " << std::strerror(errno) << std::endl;
            close(sock_fd);
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

            if(( bytes_received = sctp_recvmsg(sock_fd,buffer,sizeof(buffer), nullptr, nullptr, nullptr, nullptr)) < 0){
                std::cerr << "Recv msg: " << std::strerror(errno) << std::endl;
                close(sock_fd);
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

    close(sock_fd);
    return 0;
}