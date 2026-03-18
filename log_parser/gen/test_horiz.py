import pandas as pd
import seaborn as sb
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle

from setup import open_df

data = open_df("input_u2u_2_actors.csv").copy()

# Clean operation labels if needed
data["op"] = data["op"].replace({
    "MOV": "Move",
    "CPY": "Copy",
})

op_order = ["Move", "Copy"]
size_order = [1, 1000, 10000, 25000, 50000, 100000, 250000]

# Force the exact message sizes and ordering
data["size"] = pd.to_numeric(data["size"], errors="coerce").astype("Int64")
data = data[data["size"].isin(size_order)].copy()
data["size"] = pd.Categorical(data["size"], categories=size_order, ordered=True)

# Make row order match bar draw order
data["op"] = pd.Categorical(data["op"], categories=op_order, ordered=True)
data = data.sort_values(["size", "op"]).reset_index(drop=True)

plt.figure(figsize=(10, 6))

ax = sb.barplot(
    data=data,
    y="size",
    x="mean_us",
    hue="op",
    hue_order=op_order,
    order=size_order,
    palette="Pastel1",
    errorbar=None,
)

ax.legend(title="Method")

bars: list[Rectangle] = [
    bar
    for container in ax.containers
    for bar in container
    if isinstance(bar, Rectangle)
]

for bar, err in zip(bars, data["std_us"].to_numpy()):
    x = bar.get_width()
    y = bar.get_y() + bar.get_height() / 2

    ax.errorbar(
        x=x,
        y=y,
        xerr=err,
        fmt="none",
        ecolor="black",
        capsize=3,
    )

    ax.text(
        x + err + 0.5,
        y,
        f"{x:.1f}",
        va="center",
        ha="left",
        fontsize=9,
        )

plt.xlabel("Mean duration (µs)")
plt.ylabel("Message Size")
plt.tight_layout()
plt.show()