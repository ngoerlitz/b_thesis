import numpy as np
import pandas as pd
import seaborn
import seaborn as sns
import matplotlib.pyplot as plt

from gen.setup import apply_basic_settings, open_df

SIZES = [1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000]

# Load data
k2k_data = open_df("k2k_2_act.csv", SIZES).copy()
k2u_data = open_df("k2u_2_act.csv", SIZES).copy()
u2u_data = open_df("u2u_2_act.csv", SIZES).copy()

# Compute throughput in kB/s
for df in [k2k_data, k2u_data, u2u_data]:
    df["throughput_kB_s"] = (1e6 / df["mean_us"]) * (df["size"] / 1000)
    df["throughput_GB_s"] = df["size"] / df["mean_us"] / 1000

# Add context labels
k2k_data["context"] = "Kernel to Kernel"
k2u_data["context"] = "Kernel to User"
u2u_data["context"] = "User to User"

# Combine all data
data = pd.concat([k2k_data, k2u_data, u2u_data], ignore_index=True)

# Combined legend label
data["legend_label"] = data["context"] + " / " + data["op_label"]

plt.figure(figsize=(12, 6.75))
ax = plt.gca()

palette = sns.color_palette("Pastel1", 15)
palette = [palette[i] for i in [0, 1, 2, 3, 4, 6]]

labels = data["legend_label"].drop_duplicates().to_list()

for (label, color) in zip(labels, palette):
    subset = data[data["legend_label"] == label].sort_values("size")

    x = subset["size"].to_numpy()
    y = subset["throughput_GB_s"].to_numpy()

    seaborn.regplot(
        x=x,
        y=y,
        ax=ax,
        label=label,
        color=color,
        order=2,
        scatter_kws={
            "s": 40,
            "edgecolor": "black",
            "zorder": 3,
            "linewidths": 0.8,
        },
        line_kws={
            "linewidth": 2,
            "zorder": 2,
        },
        ci=None,
    )

# --- Use size for ticks, label with size_label ---
tick_data = (
    data.sort_values("size")
    .drop_duplicates("size")[["size", "size_label"]]
)

# --- Fixed x ticks every 50 kB ---
xticks = np.arange(0, 300000, 50000)  # up to 256kB + margin
ax.set_xticks(xticks)

# Format labels as kB
ax.set_xticklabels([f"{int(x/1000)} kB" for x in xticks])

apply_basic_settings(ax)

ax.yaxis.grid(True, linestyle="--", linewidth=0.5, alpha=0.4)
ax.set_axisbelow(True)

plt.ylabel("Throughput (GB/s)")
plt.xlabel("Message Size")
plt.legend(title="Method", loc="upper left", markerscale=1.3, handleheight=1.2)

plt.savefig("out/message_throughput.pdf", bbox_inches="tight", pad_inches=0)