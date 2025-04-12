import pandas as pd
import sys
import matplotlib.pyplot as plt

if len(sys.argv) != 2:
    print(f"Usage {sys.argv[0]} <path to stats csv>")

CSV_PATH = sys.argv[1]
df = pd.read_csv(CSV_PATH)
df = df.iloc[:-1]

plt.plot(df.index,df["Throughput/s"],label = "Throughput/request",marker = ".")

plt.title("Average throughput per  type")
plt.xlabel("Request index")
plt.ylabel("Average throughput/s")
plt.grid(True)

plt.show()