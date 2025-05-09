import time
from types import SimpleNamespace
import sctp
import socket
import select


BYTE = 1
KILOBYTE = BYTE * 1024
MEGABYTE = KILOBYTE * 1024

class SctpHttpClient:
    def __init__(self,host,port):
        """
        Creates an ipv4 one-to-one sctp socket and connects to the remote peer.
        :param host: Ipv4 address of the remote peer.
        :param port: Port number of the remote peer.
        """
        self.sctp_sock = sctp.sctpsocket_tcp(socket.AF_INET)
        self.sctp_sock.connect((host,port))
        self.sctp_sock.initparams.max_instreams = 1
        self.sctp_sock.initparams.max_instreams = 1

    def get(self,path: str):
        """
        Make a simple HTTP GET request.
        :param path: File to fetch
        :return: Namespace containing the elaped time of the request.
        """
        http_header = f"GET {path} HTTP/1.1\r\nHost: sctp\r\n\r\n".encode()

        # Send the request and start the timer
        start = time.perf_counter()
        self.sctp_sock.sctp_send(http_header,stream=0)

        # Get the header message
        select.select([self.sctp_sock], [], [],None)
        _, _, response_header, _ = self.sctp_sock.sctp_recv(16 * KILOBYTE)


        # Parse the header and get the content size
        headers = response_header.decode().split('\r\n')
        content_length_header = [header for header in headers if "content-length" in header.lower()][0]
        content_length = int(content_length_header.split(' ')[1])

        current_size = 0

        # Read chunks of the file until it is received as a whole
        while True:

            select.select([self.sctp_sock], [], [], None)
            _, _, data, _ = self.sctp_sock.sctp_recv(16 * KILOBYTE)


            data_size = len(data)
            if data_size == 0:
                break

            current_size += data_size


            if current_size == content_length:
                break

        # End the timer
        elapsed = (time.perf_counter() - start) * 1000

        return SimpleNamespace(
            status_code=200,
            content_length=content_length,
            elapsed=elapsed
        )

    def close(self):
        self.sctp_sock.close()