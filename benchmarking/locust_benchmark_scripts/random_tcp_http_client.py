import time
from types import SimpleNamespace

import socket

BYTE = 1
KILOBYTE = BYTE * 1024
MEGABYTE = KILOBYTE * 1024

class TcpHttpClient:
    def __init__(self, host, port):
        self.host = host
        self.tcp_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.tcp_sock.connect((host, port))

    def recv_response_header(self):
        buffer = b""
        while b"\r\n\r\n" not in buffer:
            chunk = self.tcp_sock.recv(4 * KILOBYTE)
            if not chunk:
                break
            buffer += chunk
        return buffer

    def recv_exact(self, n):
        current_size = 0
        while current_size < n:
            chunk = self.tcp_sock.recv(n - current_size)
            if not chunk:
                raise EOFError(f"Socket could not read {n} bytes")
            current_size += len(chunk)
        return current_size

    def get(self, path: str):
        http_request = (
            f"GET {path} HTTP/1.1\r\n"
            f"Host: {self.host}\r\n"
            f"Connection: Keep-Alive\r\n"
            f"\r\n"
        ).encode()

        start = time.perf_counter()
        self.tcp_sock.sendall(http_request)

        response = self.recv_response_header()
        header_part, leftover = response.split(b"\r\n\r\n", 1)

        # Parse the header
        headers = header_part.decode().split("\r\n")
        content_length = None
        for line in headers:
            if "content-length" in line.lower():
                content_length = int(line.split(":")[1].strip())
                break

        if content_length is None:
            raise ValueError("Missing Content-Length header")

        # Read the body file
        current_size = len(leftover)

        if current_size < content_length:
            self.recv_exact(content_length - current_size)

        elapsed = (time.perf_counter() - start) * 1000

        return SimpleNamespace(
            status_code=200,
            content_length=content_length,
            elapsed=elapsed
        )

    def close(self):
        self.tcp_sock.close()


from locust import User, task, events
from urllib.parse import urlparse
from utils.files import *

class TcpHttpUser(User):

    def on_start(self):
        parsed_url = urlparse(self.host)
        host = parsed_url.hostname
        port = parsed_url.port
        self.client = TcpHttpClient(host,port)

    @task
    def random_file_request(self):
        """
        Chooses a random file from the root directory according to the files.py file.
        Makes the request and sends the metadata about the request to the locust statistics runtime.
        :return:
        """
        file_path = choose_dir_file(choose_dir())
        parts = file_path.split(os.sep)
        file_path = os.sep + os.path.join(*parts[2:])

        try:

            metadata = self.client.get(file_path)
            events.request.fire(
                request_type="TCP",
                name=f"GET {file_path}",
                response_time = metadata.elapsed,
                response_length = metadata.content_length,
                exception = None,
            )

        except Exception as e:
            events.request.fire(
                request_type="TCP",
                name=f"GET {file_path}",
                response_time=0,
                response_length=0,
                exception=e,
            )

    def on_stop(self):
        self.client.close()