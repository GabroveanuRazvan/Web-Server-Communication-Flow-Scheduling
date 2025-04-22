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

RUN_TITLE = "Test"
NUM_RUNS = 4
LOCUST_FILE_PATH = "./locust_benchmark_scripts/locust_random_client.py"
NUM_USERS = 6
SPAWN_RATE = NUM_USERS
HOST = "http://192.168.100.118:7878"
RUN_TIME = "60s"
CSV_ROOT = "./results_headless"
RESULT_DIR_NAME = f"{RUN_TITLE}{{}}_U{{}}_T{{}}"
CSV_FILE_NAME = f"{RUN_TITLE}{{}}"

USE_FULL_CSV_HISTORY = True
THROUGHPUT_PLOT_FILE_NAME = f"{RUN_TITLE}{{}}_throughput.png"
THROUGHPUT_ALL_FILE_NAME = f"{RUN_TITLE}_throughput_all.png"
AVG_THROUGHPUT_FILE_NAME = f"{RUN_TITLE}_throughput_avg.png"

os.makedirs(CSV_ROOT, exist_ok=True)


fig_global,plt_global = plt.subplots()
fig_avg_throughput,plt_avg_throughput = plt.subplots()
runs = list(range(1,NUM_RUNS+1))
avg_throughput = []

for run in runs:
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
    df.to_csv(stats_file_path,index=False)

    current_avg_throughput = df[df["Name"] == "Aggregated"].iloc[0]["Throughput/s"]
    avg_throughput.append(current_avg_throughput)

    if USE_FULL_CSV_HISTORY:
        # Drop rows having N/A values
        df = pd.read_csv(history_file_path).dropna(subset=["50%"])
        # Keep only the aggregated stats
        df = df[df['Name'] == "Aggregated"]

        # Normalize the timestamp
        seconds_column = df["Timestamp"] - df["Timestamp"].iloc[0]

        # Convert the timestamp to datetime format
        df["Timestamp"] = df["Timestamp"].apply(lambda t: datetime.fromtimestamp(t).strftime("%H:%M:%S"))

        # Compute the throughput
        df = add_throughput_history(df)
        df.to_csv(history_file_path, index=False)
        fig_local, plt_local = plt.subplots()

        # Plot the results of the throughput in time on the local figure
        plt_local.plot(seconds_column, df["Throughput/s"], label="Throughput/sec", marker=".")

        plt_local.set_title("Average throughput in time")
        plt_local.set_xlabel("Timestamp in seconds")
        plt_local.set_ylabel("Throughput in bytes")
        plt_local.legend()

        plot_file_path = os.path.join(CSV_ROOT,result_dir, THROUGHPUT_PLOT_FILE_NAME.format(run))
        fig_local.savefig(plot_file_path)

        # Add the throughput results to the global figure
        plt_global.plot(seconds_column, df["Throughput/s"], label=str(run), marker=".")
        plt_global.set_title("Average throughput in time")
        plt_global.set_xlabel("Timestamp in seconds")
        plt_global.set_ylabel("Throughput in bytes")
        plt_global.legend()

# Save the global plot
global_plot_file_path = os.path.join(CSV_ROOT, THROUGHPUT_ALL_FILE_NAME)
fig_global.savefig(global_plot_file_path)

# Compute the average throughput across runs plot
plt_avg_throughput.plot(runs,avg_throughput,label= "Average throughput across runs", marker = '.')
plt_avg_throughput.set_title("Average throughput across runs")
plt_avg_throughput.set_xlabel("Run index")
plt_avg_throughput.set_ylabel("Throughput in bytes")

plot_file_path = os.path.join(CSV_ROOT, AVG_THROUGHPUT_FILE_NAME)
fig_avg_throughput.savefig(plot_file_path)


# locust --headless --locustfile ./locust_benchmark_scripts/locust_random_client.py --users 6 --spawn-rate 6.0 --host http://192.168.1.144:7878 --run-time 60s --skip-log