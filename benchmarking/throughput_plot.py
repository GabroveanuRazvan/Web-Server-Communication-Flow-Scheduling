import pandas as pd
import matplotlib.pyplot as plot

CSV_PATH = "./results_headless/ConnSchedPiServer/ConnSchedPiServer1_U6_T15m/ConnSchedPiServer1_stats_history.csv"
PLOT_PATH = "./results_headless/ConnSchedPiServer/ConnSchedPiServer_Plots/throughput_requests.png"
df = pd.read_csv(CSV_PATH)

fig,plt = plot.subplots()

plt.plot(df["Total Request Count"],df["Throughput/s"],label = "Throughput", marker = ".")

plt.set_title("Throughput in time")
plt.set_xlabel("Requests")
plt.set_ylabel("Throughput in bytes")

fig.savefig(PLOT_PATH)