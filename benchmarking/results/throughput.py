import csv

def calculate_throughput(csv_path):
    total_throughput = 0
    print("Throughput per endpoint:")

    with open(csv_path, newline='') as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            try:
                avg_bytes = float(row["Average Content Size"])
                rps = float(row["Requests/s"])
                throughput = avg_bytes * rps  # bytes/sec
                total_throughput += throughput

                print(f"{row['Name']:20} → {throughput / 1024:.2f} KB/s")

            except ValueError:
                continue  # skip header/footer rows

    print(f"\nTOTAL THROUGHPUT: {total_throughput / 1024:.2f} KB/s ({total_throughput / 1024 / 1024:.2f} MB/s)")

# Ex: calculate_throughput("locust_stats.csv")
if __name__ == "__main__":
    calculate_throughput("./results_stats.csv")
