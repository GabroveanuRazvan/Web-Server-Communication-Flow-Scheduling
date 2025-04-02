from locust import HttpUser, task, between

class WebUser(HttpUser):
    wait_time = between(1, 5)

    @task(1)
    def index_page(self):
        self.client.get("/")

    @task(3)
    def image(self):
        self.client.get("/images_4k/4k1.jpg")