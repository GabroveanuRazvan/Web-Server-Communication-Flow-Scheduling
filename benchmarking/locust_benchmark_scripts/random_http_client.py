from locust import HttpUser, task, constant

from utils.files import *

class WebUser(HttpUser):
    wait_time = constant(0)

    @task
    def random_file_request(self):
        file_path = choose_dir_file(choose_dir())
        parts = file_path.split(os.sep)
        file_path = os.sep + os.path.join(*parts[2:])
        self.client.get(file_path)

#http://127.0.0.1:7878