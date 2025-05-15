from locust import User, task, events
from urllib.parse import urlparse
from utils.files import *
from utils.tcp import TcpHttpClient
import os
import json

REQUESTS_PATH = "./requests"
REQUESTS_FILE_NAME = "requests"

class TcpHttpUser(User):
    requests_index = 0


    def on_start(self):

        request_file_path = os.path.join(REQUESTS_PATH, f"{REQUESTS_FILE_NAME}_{TcpHttpUser.requests_index}.json")

        parsed_url = urlparse(self.host)
        host = parsed_url.hostname
        port = parsed_url.port
        self.client = TcpHttpClient(host,port)

        with open(request_file_path) as file:
            self.requests = json.load(file)

        self.request_count = len(self.requests)
        self.req_index = 0

    @task
    def random_file_request(self):
        file_path = self.requests[self.req_index]
        self.req_index+=1

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

        if self.req_index == self.request_count:
            self.environment.runner.quit()

    def on_stop(self):
        self.client.close()