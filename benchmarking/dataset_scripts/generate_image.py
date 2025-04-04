
import numpy as np
import os
import math
from io import BytesIO
from PIL import Image


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

height = 25
width = 25

growth_rate = 1.01

save_path = "./benchmark_dataset"

# Sorted, non overlapping intervals
intervals = [(5 * KILOBYTE, 10 * KILOBYTE),
             (30 * KILOBYTE, 50 * KILOBYTE),
             (100 * KILOBYTE, 300 * KILOBYTE),
             (1 * MEGABYTE, 5 * MEGABYTE)]



# Generate a directory for each interval in the given file root
intervals_paths = [os.path.join(save_path,serialize_interval(interval)) for interval in intervals]

for path in intervals_paths:
    os.makedirs(path, exist_ok=True)

interval_index = 0
image_count = 1

# print(interval_index)

while interval_index < len(intervals):
    interval = intervals[interval_index]

    # Generate an image and get its PNG size
    img = np.random.randint(0,256,(height,width,3),dtype=np.uint8)
    image_buffer = BytesIO()
    img = Image.fromarray(img)
    img.save(image_buffer, format='PNG')
    image_size = image_buffer.tell()

    # Store the image in the current interval if its size fits
    if interval[0] <= image_size < interval[1]:
        image_file_name = f"{image_count}.png"
        image_path = os.path.join(intervals_paths[interval_index],image_file_name)
        img.save(image_path, format='PNG')
        image_count += 1

    # Go to the next interval if the image size is too big
    elif image_size >= interval[1]:
        interval_index += 1
        image_count = 1
        # print(interval_index)

    # Multiplicative growth rate for the images
    print(height,width)
    height = math.ceil(height * growth_rate)
    width = math.ceil(width * growth_rate)
