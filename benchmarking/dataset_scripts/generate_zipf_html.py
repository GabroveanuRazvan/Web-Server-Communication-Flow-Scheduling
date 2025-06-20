import numpy as np
import os

DATASET_DIR_PATH  = "../zipf_dataset"
TEMPLATE_FILE_PATH = "./template_raw.html"

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

FILE_COUNT = 2000


os.makedirs(DATASET_DIR_PATH, exist_ok=True)

# Compute zipf probabilities using the zipf exponent
zipf_s = 1.0
ranks = np.arange(1, len(SIZE_RANGES) + 1)
weights = 1 / np.power(ranks, zipf_s)
probabilities = weights / np.sum(weights)

# Using the probabilities choose the indexes of each range and generate a random file size for each range
chosen_indexes = np.random.choice(np.arange(0,len(SIZE_RANGES)), size = FILE_COUNT, p = probabilities)
chosen_sizes = np.array([np.random.randint(low = SIZE_RANGES[idx][0],high = SIZE_RANGES[idx][1]) for idx in chosen_indexes])

# Build and save each file
for idx in range(0,FILE_COUNT):
    file_name = f"{idx}.html"
    file_path = os.path.join(DATASET_DIR_PATH,file_name)
    file_size = chosen_sizes[idx]

    with open(TEMPLATE_FILE_PATH) as template:
        template_file = template.read()

    file_content = template_file.format(TEXT = "T" * file_size)
    del template_file

    with open(file_path, "w") as file:
        file.write(file_content)

    del file_content

