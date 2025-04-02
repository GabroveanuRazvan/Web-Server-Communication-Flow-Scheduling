from locust import HttpUser, task, between

class WebUser(HttpUser):
    wait_time = between(1, 5)

    @task
    def index_page(self):
        self.client.get("/")