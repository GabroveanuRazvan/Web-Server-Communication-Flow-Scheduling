import time

from locust import User, task, events
from utils.files import *
import json

class StatisticsUser(User):

    def on_start(self):
        with open(COMPUTE_STATS_PATH, "r") as f:
            self.requests = json.load(f)

    @task
    def random_file_request(self):


        for event in self.requests:

            events.request.fire(
                request_type=event["request_type"],
                name=event["name"],
                response_time=event["response_time"],
                response_length=event["response_length"],
                exception=None,
            )

        time.sleep(5)
        self.environment.runner.quit()


