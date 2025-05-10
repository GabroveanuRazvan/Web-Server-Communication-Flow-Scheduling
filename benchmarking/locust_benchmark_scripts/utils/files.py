import os
import numpy as np
import pickle as pkl

# Used in one file benchmarking
FILE_TO_FETCH_PATH = "/3.00M-6.00M/49.html"

# Used to fetch a random file from this root
ROOT = "./benchmark_raw_dataset"


def walk_dir(root: str):
    file_paths = [os.path.join(root, file) for file in os.listdir(root)]

    new_files = []
    for file in file_paths:
        if os.path.isdir(file):
            new_files += walk_dir(file)
        else:
            new_files.append(file)

    return new_files


DIRS = [path.removeprefix(ROOT) for path in walk_dir(ROOT)]


def choose_file():
    num_dirs = len(DIRS)
    if num_dirs == 0:
        raise Exception("No benchmark dataset")

    index = np.random.randint(num_dirs)
    return DIRS[index]


# Generate or load from a pickle file the list of requests
NUM_REQUESTS = 10000
REQUESTS_LIST_PATH = f"./requests_list_{NUM_REQUESTS}.pkl"

def get_requests() -> list[str]:

    if os.path.exists(REQUESTS_LIST_PATH):
        return pkl.load(open(REQUESTS_LIST_PATH, "rb"))

    requests = [choose_file() for _ in range(NUM_REQUESTS)]
    pkl.dump(requests, open(REQUESTS_LIST_PATH, "wb"))
    return requests


if __name__ == "__main__":
    print(walk_dir(ROOT))
    print(DIRS)
