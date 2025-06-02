import random
import os
import shutil

def clear_dir(dir_path: str):
    """
    Recursively remove all contents of a directory
    :param dir_path: Path to the directory
    :return: None
    """
    if not os.path.isdir(dir_path):
        return

    for element in os.listdir(dir_path):
        file_path = os.path.join(dir_path,element)

        if os.path.isfile(file_path) or os.path.islink(file_path):
            os.unlink(file_path)

        elif os.path.isdir(file_path):
            shutil.rmtree(file_path)

def serialize_interval(interval: (int,int)):
    """
     Takes an interval and returns a serialized name, in kilobytes or megabytes units.
    :param interval: A tuple representing the interval
    :return: A name for the given interval
    """
    left,right = interval

    unit_sign = 'K'
    unit_used = KILOBYTE

    if right / MEGABYTE >= 1.0:
        unit_sign = 'M'
        unit_used = MEGABYTE

    left /= unit_used
    right /= unit_used

    return f"{left:.2f}{unit_sign}-{right:.2f}{unit_sign}"

"""
Parameters used by the script
"""

BYTE = 1
KILOBYTE = 1024 * BYTE
MEGABYTE = 1024 * KILOBYTE

NUM_FILES_PER_INTERVAL = 50
TEMPLATE_PATH = "./template_raw.html"
SAVE_PATH = "../raw_dataset"

# Clear the contents of the directory
clear_dir(SAVE_PATH)

# Sorted, non overlapping intervals
INTERVALS = [(5 * KILOBYTE, 10 * KILOBYTE),
             (30 * KILOBYTE, 50 * KILOBYTE),
             (60 * KILOBYTE, 90 * KILOBYTE),
             (100 * KILOBYTE, 300 * KILOBYTE),
             (400 * KILOBYTE, 700 * KILOBYTE),
             (1 * MEGABYTE, 3 * MEGABYTE),
             (3 * MEGABYTE, 6 * MEGABYTE)]


with open(TEMPLATE_PATH,'r') as f:
    TEMPLATE_CONTENT = f.read()


# Generate a directory for each interval in the given file root
intervals_paths = [os.path.join(SAVE_PATH,serialize_interval(interval)) for interval in INTERVALS]

for path in intervals_paths:
    os.makedirs(path, exist_ok=True)


for interval in INTERVALS:
    interval_path = os.path.join(SAVE_PATH,serialize_interval(interval))

    for index in range(NUM_FILES_PER_INTERVAL):
        content_size = random.randint(interval[0], interval[1])
        noise = 'T' * content_size
        content = TEMPLATE_CONTENT.format(TEXT = noise)

        file_path = os.path.join(interval_path,f"{index}.html")
        with open(file_path,"w") as f:
            f.write(content)


