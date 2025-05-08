from utils.sctp import SctpHttpClient
from locust import User, task, events
from urllib.parse import urlparse
from utils.files import FILE_TO_FETCH_PATH

class SctpHttpUser(User):

    def on_start(self):
        parsed_url = urlparse(self.host)
        host = parsed_url.hostname
        port = parsed_url.port
        self.client = SctpHttpClient(host,port)

    @task
    def one_file_request(self):
        """
        Makes the same requests continuously.
        Makes the request and sends the metadata about the request to the locust statistics runtime.
        :return:
        """

        try:

            metadata = self.client.get(FILE_TO_FETCH_PATH)
            events.request.fire(
                request_type="STCP",
                name=f"GET {FILE_TO_FETCH_PATH}",
                response_time = metadata.elapsed,
                response_length = metadata.content_length,
                exception = None,
            )

        except Exception as e:
            events.request.fire(
                request_type="STCP",
                name=f"GET {FILE_TO_FETCH_PATH}",
                response_time=0,
                response_length=0,
                exception=e,
            )

    def on_stop(self):
        self.client.close()
