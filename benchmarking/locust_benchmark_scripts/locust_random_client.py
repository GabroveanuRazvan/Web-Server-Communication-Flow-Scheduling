from locust import HttpUser, task, between
import os
import numpy as np

root = "./benchmark_dataset"
dirs = [os.path.join(root,current_dir) for current_dir in os.listdir(root)]


def choose_dir():
    num_dirs = len(dirs)
    if num_dirs == 0:
        raise Exception("No benchmark dataset")

    index = np.random.randint(num_dirs)
    return dirs[index]

def choose_dir_file(dir : str):
    files = [os.path.join(dir,file_name) for file_name in os.listdir(dir)]
    num_files = len(files)
    if num_files == 0:
        raise Exception("Empty dir in benchmark dataset")

    index = np.random.randint(num_files)
    return files[index]



class WebUser(HttpUser):
    wait_time = between(0, 0)

    @task
    def random_file(self):
        file_path = choose_dir_file(choose_dir())
        parts = file_path.split(os.sep)
        file_path = os.sep + os.path.join(*parts[2:])
        self.client.get(file_path)

if __name__ == "__main__":
    import os
    path = "/a/b/c"
    parts = path.split(os.sep)
    new_path = os.sep + os.path.join(*parts[2:])  # păstrăm '/' în față
    print(parts)

#http://127.0.0.1:7878