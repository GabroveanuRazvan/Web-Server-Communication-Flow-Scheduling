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

    def on_stop(self):
        self.client.close()
