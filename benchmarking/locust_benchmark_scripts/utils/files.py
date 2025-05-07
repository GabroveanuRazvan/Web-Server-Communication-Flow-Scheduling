import os
import numpy as np

root = "./benchmark_raw_dataset"
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