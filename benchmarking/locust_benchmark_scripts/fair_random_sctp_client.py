
import sys
import time

from utils.sctp import SctpHttpClient
from locust import User, task, events
from urllib.parse import urlparse
from utils.files import *

class SctpHttpUser(User):

    def on_start(self):
        parsed_url = urlparse(self.host)
        host = parsed_url.hostname
        port = parsed_url.port
        self.client = SctpHttpClient(host,port)
        self.requests = get_requests()
        self.request_count = NUM_REQUESTS
        self.req_index = 0

    @task
    def random_file_request(self):
        
        file_path = self.requests[self.req_index]
        self.req_index+=1

        try:

            metadata = self.client.get(file_path)
            events.request.fire(
                request_type="STCP",
                name=f"GET {file_path}",
                response_time = metadata.elapsed,
                response_length = metadata.content_length,
                exception = None,
            )

        except Exception as e:
            events.request.fire(
                request_type="STCP",
                name=f"GET {file_path}",
                response_time=0,
                response_length=0,
                exception=e,
            )
        
        if self.req_index == self.request_count:
            self.environment.runner.quit()


    def on_stop(self):
        self.client.close()
