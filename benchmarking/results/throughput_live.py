import pandas as pd
import matplotlib.pyplot as plt
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler
import time

CSV_PATH = "results_stats.csv"
ELAPSED_TIME = []  # seconds
THROUGHPUT_KB = []  # KB/sec

start_time = time.time()

def compute_throughput():
    try:
        df = pd.read_csv(CSV_PATH)
        total_bytes = 0
        total_requests = 0

        for _, row in df.iterrows():
            try:
                avg_bytes = float(row["Average Content Size"])
                requests = int(row["Requests"])
                total_bytes += avg_bytes * requests
                total_requests += requests
            except:
                continue

        elapsed = time.time() - start_time
        throughput = total_bytes / elapsed / 1024  # KB/s
        return elapsed, throughput
    except:
        return None, None

def update_plot():
    plt.clf()
    plt.plot(ELAPSED_TIME, THROUGHPUT_KB, label="Throughput (KB/s)", color='blue')
    plt.xlabel("Timp (s)")
    plt.ylabel("Throughput (KB/s)")
    plt.title("Evoluția throughput-ului în timp")
    plt.grid(True)
    plt.legend()
    plt.pause(0.5)

class CSVHandler(FileSystemEventHandler):
    def on_modified(self, event):
        if event.src_path.endswith(CSV_PATH):
            elapsed, throughput = compute_throughput()
            if elapsed and throughput:
                ELAPSED_TIME.append(elapsed)
                THROUGHPUT_KB.append(throughput)
                update_plot()

if __name__ == "__main__":
    print("Monitorizare în timp real...")
    event_handler = CSVHandler()
    observer = Observer()
    observer.schedule(event_handler, path=".", recursive=False)
    observer.start()

    plt.ion()
    plt.figure()

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()
