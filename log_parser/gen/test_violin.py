import pandas as pd
import seaborn as sb
import matplotlib.pyplot as plt

from gen.setup import open_df_raw

data = open_df_raw("input_u2u_2_actors.csv").copy()

data["op"] = data["op"].replace({
    "MOV": "Move",
    "CPY": "Copy",
})

size_order = [1, 1000, 10000, 25000, 50000, 100000]
data["size"] = pd.to_numeric(data["size"], errors="coerce").astype("Int64")
data = data[data["size"].isin(size_order)].copy()

data["size"] = pd.Categorical(
    data["size"],
    categories=size_order,
    ordered=True
)

op_order = ["Move", "Copy"]

plt.figure(figsize=(10, 6))

ax = sb.violinplot(
    data=data,
    y="size",
    x="duration_us",
    hue="op",
    hue_order=op_order,
    palette="Pastel1",
    cut=0,
    inner='box'
)

handles, labels = ax.get_legend_handles_labels()
ax.legend(handles[:2], labels[:2], title="Method")

plt.xlabel("Duration (µs)")
plt.ylabel("Message Size")
plt.tight_layout()
plt.show()