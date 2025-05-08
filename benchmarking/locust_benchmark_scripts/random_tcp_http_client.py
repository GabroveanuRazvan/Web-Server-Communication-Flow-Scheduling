import time
from types import SimpleNamespace

import socket

BYTE = 1
KILOBYTE = BYTE * 1024
MEGABYTE = KILOBYTE * 1024

class TcpHttpClient:
    def __init__(self,host,port):
        self.tcp_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.tcp_sock.connect((host, port))


    def get(self,path: str):
        http_header = f"GET {path} HTTP/1.1\r\nHost: sctp\r\n\r\n".encode()

        start = time.perf_counter()
        self.tcp_sock.sendall(http_header)
        reader = self.tcp_sock.makefile("rb")

        while True:
            line = reader.readline().decode()
            if line == "\r\n":
                break

            if "content-length" in line.lower():
                content_length = int(line.split(' ')[1])

        current_size = 0

        while True:

            data = self.tcp_sock.recv(16 * KILOBYTE)
            data_size = len(data)

            if data_size == 0:
                break

            current_size += data_size

            if current_size == content_length:
                break

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