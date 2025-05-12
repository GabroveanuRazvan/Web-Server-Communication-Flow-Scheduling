import time

from locust import User, task, events
from utils.files import *
import json
import subprocess

EXECUTABLE_PATH = "../sctp_benchmarking_script/target/release/sctp_benchmarking_script"

class StatisticsUser(User):

    def on_start(self):
        pass

    @task
    def random_file_request(self):
        bench_client_process = subprocess.Popen([EXECUTABLE_PATH],stdout = subprocess.PIPE)

        for line in bench_client_process.stdout:
            data = json.loads(line)

            events.request.fire(
                request_type=data["request_type"],
                name=data["name"],
                response_time=data["response_time"],
                response_length=data["response_length"],
                exception=None,
            )


        time.sleep(5)
        self.environment.runner.quit()



if __name__ == "__main__":
    bench_client_process = subprocess.Popen([EXECUTABLE_PATH], stdout=subprocess.PIPE)

    for line in bench_client_process.stdout:
        data = json.loads(line)
        print(data)