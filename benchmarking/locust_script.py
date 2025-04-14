import subprocess
import pandas as pd
import matplotlib.pyplot as plt
import os
from datetime import datetime

def add_throughput_stats(df: pd.DataFrame) -> pd.DataFrame:
    df["Throughput/s"] = (df["Requests/s"] * df["Average Content Size"]).round(6)
    return df

def add_throughput_history(df: pd.DataFrame) -> pd.DataFrame:
    df["Throughput/s"] = (df["Requests/s"] * df["Total Average Content Size"]).round(6)
    return df


NUM_RUNS = 3
LOCUST_FILE_PATH = "./locust_benchmark_scripts/locust_random_client.py"
NUM_USERS = 6
SPAWN_RATE = NUM_USERS
HOST = "http://192.168.1.143:7878"
RUN_TIME = "10s"
CSV_ROOT = "./results_headless"
RESULT_DIR_NAME = f"Run{{}}_U{{}}_T{{}}"
CSV_FILE_NAME = f"Run{{}}"

USE_FULL_CSV_HISTORY = True
THROUGHPUT_PLOT_FILE_NAME = f"Run{{}}_throughput.png"


os.makedirs(CSV_ROOT, exist_ok=True)


for run in range(1,NUM_RUNS+1):
    print(f"Run {run} started.")

    # Build the locust args and resolve the used paths
    result_dir = RESULT_DIR_NAME.format(run, NUM_USERS, RUN_TIME)
    csv_file_name = CSV_FILE_NAME.format(run)
    results_path = os.path.join(CSV_ROOT,result_dir,csv_file_name)

    stats_file_path = results_path + "_stats.csv"
    history_file_path = results_path + "_stats_history.csv"


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

    if USE_FULL_CSV_HISTORY:
        locust_args.append("--csv-full-history")

    # Ramp up the number of users for each benchmarking session
    NUM_USERS += 1
    SPAWN_RATE = NUM_USERS

    # Run the locust benchmarking test
    child = subprocess.run(locust_args,capture_output=True,text=True)

    print(f"Run {run} ended.")
    print(f"Computing stats...")

    print(child.stdout)
    print(child.stderr)

    # Compute the throughput for each stat
    df = pd.read_csv(stats_file_path)
    df = add_throughput_stats(df)
    df.to_csv(stats_file_path, index=False)

    if USE_FULL_CSV_HISTORY:
        # Drop rows having N/A values
        df = pd.read_csv(history_file_path).dropna(subset=["50%"])
        # Keep only the aggregates stats
        df = df[df['Name'] == "Aggregated"]

        seconds_column = df["Timestamp"] - df["Timestamp"].iloc[0]

        # Convert the timestamp to datetime format
        df["Timestamp"] = df["Timestamp"].apply(lambda t: datetime.fromtimestamp(t).strftime("%H:%M:%S"))
        # Compute the throughput
        df = add_throughput_history(df)
        df.to_csv(history_file_path, index=False)

        # Plot the results in time
        plt.plot(seconds_column, df["Throughput/s"], label="Throughput/sec", marker=".")

        plt.title("Average throughput in time")
        plt.xlabel("Timestamp in seconds")
        plt.ylabel("Throughput in bytes")

        plot_file_path = os.path.join(CSV_ROOT,result_dir, THROUGHPUT_PLOT_FILE_NAME.format(run))
        plt.savefig(plot_file_path)

        plt.clf()




# locust --headless --locustfile ./locust_benchmark_scripts/locust_random_client.py --users 6 --spawn-rate 6.0 --host http://192.168.1.144:7878 --run-time 60s --skip-log