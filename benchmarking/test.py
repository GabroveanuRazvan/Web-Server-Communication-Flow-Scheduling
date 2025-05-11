import pickle as pkl
import json

l = pkl.load(open("./requests_list_10000.pkl", "rb"))

json.dump(l, open("../tcp_benchmarking_script/requests_list_10000.json", "w"))