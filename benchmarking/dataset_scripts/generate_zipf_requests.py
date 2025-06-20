import json
import os
import numpy as np

DATASET_ROOT = "../zipf_dataset"
REQUESTS_PATH = "../requests/requests_zipf_.json"

REQUEST_COUNT = 5000

BYTE = 1
KILOBYTE = 1024 * BYTE
MEGABYTE = 1024 * KILOBYTE

SIZE_RANGES = [

    (6 * KILOBYTE, 14 * KILOBYTE),
    (14 * KILOBYTE, 24 * KILOBYTE),
    (24 * KILOBYTE, 64 * KILOBYTE),
    (64 * KILOBYTE, 256 * KILOBYTE),
    (256 * KILOBYTE, 1 * MEGABYTE),
    (1 * MEGABYTE, 3 * MEGABYTE),
    (3 * MEGABYTE, 6 * MEGABYTE),

]

def find_range(size: int):

    for idx, current_range in enumerate(SIZE_RANGES):
        start,end = current_range

        if start <= size < end:
            return idx,current_range

    raise Exception("No range found")


range_dict = {i:[] for i in range(len(SIZE_RANGES))}
file_names = os.listdir(DATASET_ROOT)

for file_name in file_names:

    file_path = os.path.join(DATASET_ROOT, file_name)
    with open(file_path,"r") as f:
        file_size = os.stat(file_path).st_size

    idx,current_range = find_range(file_size)

    range_dict[idx].append(file_name)


# Compute zipf probabilities using the zipf exponent
zipf_s = 1.0
ranks = np.arange(1, len(SIZE_RANGES) + 1)
weights = 1 / np.power(ranks, zipf_s)
probabilities = weights / np.sum(weights)

# Using the probabilities choose the indexes of each range and generate a random file size for each range
chosen_indexes = np.random.choice(np.arange(0,len(SIZE_RANGES)), size = REQUEST_COUNT, p = probabilities)

chosen_requests = []

for idx in chosen_indexes:
    files = range_dict[idx]
    file_count = len(files)
    random_file_idx = np.random.randint(0,file_count)
    chosen_requests.append("/" + files[random_file_idx])


with open(REQUESTS_PATH,"w") as f:
    f.write(json.dumps(chosen_requests))