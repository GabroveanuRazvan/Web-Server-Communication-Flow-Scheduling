import os
import random


def flatten_dir(current_dir: str):
    """
    Walks the current directory and fetches the paths of all files in its tree.
    :param current_dir: The root directory path
    :return: List of paths
    """
    file_list = []
    files = [os.path.join(current_dir,file_name) for file_name in os.listdir(current_dir)]

    for file in files:

        if os.path.isfile(file):
            file_list.append(file)
        else:
            pass
            file_list.extend(flatten_dir(file))

    return file_list


root_dir = "../raw_dataset"
htmls_dir_name =  "html_pages"
html_dir_path = os.path.join(root_dir, htmls_dir_name)
html_template_path = "template_images.html"

img_content = """ <img src="{path}" width="100" height="100" alt="not working"> """

all_images_paths = flatten_dir(root_dir)
all_images_paths = [".." + os.sep + os.sep.join(file_path.split(os.sep)[2:])  for file_path in all_images_paths]

pages_count = 30
min_images_count = min(10,len(all_images_paths))
max_images_count = min(30,len(all_images_paths))

if not os.path.exists(root_dir) or not os.path.isdir(root_dir):
    raise FileNotFoundError(root_dir)

if not os.path.exists(html_template_path) or not os.path.isfile(html_template_path):
    raise FileNotFoundError(html_template_path)

html_template = ""

with open(html_template_path,"r") as f:
    html_template = f.read()

os.makedirs(html_dir_path,exist_ok=True)


file_contents = []

for i in range(pages_count):
    file_name = f"page_{i+1}.html"
    file_path = os.path.join(html_dir_path,file_name)

    images_content = ""

    image_count = random.randint(min_images_count,max_images_count)
    chosen_paths = random.sample(all_images_paths,image_count)

    for image_path in chosen_paths:
        current_content = img_content.format(path=image_path)
        images_content += current_content + '\n'

    current_file_content = html_template.format(IMAGES=images_content,BUTTONS="\n")

    file_contents.append((file_path,current_file_content))


for file_path,file_content in file_contents:
    with open(file_path,"w") as f:
        f.write(file_content)
