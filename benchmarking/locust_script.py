import subprocess
import pandas as pd
import os

def add_throughput(df: pd.DataFrame) -> pd.DataFrame:
    df["Throughput/s"] = (df["Requests/s"] * df["Average Content Size"]).round(6)
    return df


NUM_RUNS = 4
LOCUST_FILE_PATH = "./locust_benchmark_scripts/locust_random_client.py"
NUM_USERS = 6
SPAWN_RATE = NUM_USERS
HOST = "http://192.168.1.143:7878"
RUN_TIME = "5s"
CSV_ROOT = "./results_headless"
RESULT_DIR_NAME = f"Run{{}}_U{{}}_T{{}}"
CSV_FILE_NAME = f"Run{{}}"

os.makedirs(CSV_ROOT, exist_ok=True)


for run in range(1,NUM_RUNS+1):
    print(f"Run {run} started.")

    # Build the locust args and resolve the used paths
    result_dir = RESULT_DIR_NAME.format(run, NUM_USERS, RUN_TIME)
    csv_file_name = CSV_FILE_NAME.format(run)
    results_path = os.path.join(CSV_ROOT,result_dir,csv_file_name)

    stats_file_path = results_path + "_stats.csv"

    locust_args = ["locust",
                   "--headless",
                   "--skip-log",
                   "--locustfile",
                   LOCUST_FILE_PATH,
                   "--users",
                   str(NUM_USERS),
                   "--spawn-rate",
                   str(SPAWN_RATE),
                   "--host",
                   HOST,
                   "--run-time",
                   RUN_TIME,
                   "--csv",
                   results_path,
                   ]

    # Ramp up the number of users for each benchmarking session
    NUM_USERS += 1
    SPAWN_RATE = NUM_USERS

    # Run the locust benchmarking test
    child = subprocess.run(locust_args,capture_output=True,text=True)

    # Compute the throughput for each stat
    df = pd.read_csv(stats_file_path)
    df = add_throughput(df)
    df.to_csv(stats_file_path, index=False)

    print(f"Run {run} ended.")
    print(child.stdout)
    print(child.stderr)

# locust --headless --locustfile ./locust_benchmark_scripts/locust_random_client.py --users 6 --spawn-rate 6.0 --host http://192.168.1.144:7878 --run-time 60s --skip-log