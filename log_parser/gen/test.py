import seaborn as sb
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle

from setup import open_df

data = open_df("input_u2u_2_actors.csv").copy()

op_order = ["Move", "Copy"]

plt.figure(figsize=(10, 6))

ax = sb.barplot(
    data=data,
    x="size_label",
    y="mean_us",
    hue="op",
    hue_order=op_order,
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
    x = bar.get_x() + bar.get_width() / 2
    y = bar.get_height()
    ax.errorbar(
        x=x,
        y=y,
        yerr=err,
        fmt="none",
        ecolor="black",
        capsize=3,
    )

    ax.text(
        x,
        y + err + 0.5,   # offset so it sits above the error bar
        f"{y:.1f}",
        ha="center",
        va="bottom",
        fontsize=9
    )

plt.ylabel("Mean duration (µs)")
plt.xlabel("Message Size")
plt.show()