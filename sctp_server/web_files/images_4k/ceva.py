import os

def rename_files_in_directory():
    script_name = os.path.basename(__file__)
    files = [f for f in os.listdir('.') if os.path.isfile(f) and f != script_name]

    index = 1
    for filename in files:
        name, ext = os.path.splitext(filename)
        new_name = f"4k{index}{ext}"
        os.rename(filename, new_name)
        print(f"Renamed '{filename}' to '{new_name}'")
        index += 1

if __name__ == "__main__":
    rename_files_in_directory()
