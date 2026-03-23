import numpy as np
import seaborn
import matplotlib.pyplot as plt

from gen.setup import apply_basic_settings, open_df, bar_plot_err, line_plot_err, get_label_legend

SIZES = [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000]

data = open_df("k2u_2_act.csv", SIZES)

print(data)

plt.figure(figsize=(12, 6.75))
ax = plt.gca()

ax.axis(ymin=13, ymax=21)

palette = seaborn.color_palette("Pastel1")

fits = {}

offsets = {
    "Move": -5,
    "Copy": +5,
}

for (op, color) in zip(data["op"].unique(), palette):
    subset = data[data["op"] == op].sort_values("size")

    offset = offsets[op]

    # shifted x for plotting
    x = subset["size"].to_numpy() + offset
    y = subset["mean_us"].to_numpy()

    # fit still uses original x (important!)
    m, b = np.polyfit(subset["size"], y, 1)
    fits[op] = (m, b)

    seaborn.regplot(
        x=x,
        y=y,
        ax=ax,
        label=get_label_legend(op),
        color=color,
        scatter_kws={
            "s": 40,
            "edgecolor": "black",
            "zorder": 3,
            "linewidths": 0.8,
        },
        line_kws={"linewidth": 2, "zorder": 2},
        ci=None
    )

# --- Intersection ---
ops = list(fits.keys())
op1, op2 = ops[0], ops[1]

m1, b1 = fits[op1]
m2, b2 = fits[op2]

d1 = offsets[op1]
d2 = offsets[op2]

x_intersect = (b2 - b1 + m1*d1 - m2*d2) / (m1 - m2)
y_intersect = m1 * (x_intersect - d1) + b1

ax.axvline(x=x_intersect, linestyle="--", color="gray", linewidth=1)

ax.text(
    x_intersect + 40,
    ax.get_ylim()[0] + 0.1,
    f"{x_intersect/1000:.2f} KB",
    color="gray",
    ha="center",
    va="bottom"
)

# --- Use size for ticks, label with size_label ---
tick_data = (
    data.sort_values("size")
    .drop_duplicates("size")[["size", "size_label"]]
)

ax.set_xticks(tick_data["size"])
ax.set_xticklabels(tick_data["size_label"])

apply_basic_settings(ax)
line_plot_err(ax, data, offsets, ".2f", offset_text=True, error_bar_kws={
    "elinewidth": 0.8,
    "capthick": 0.8,
    "ecolor": "gray"
}, text_offset_mult=4)

ax.yaxis.grid(True, linestyle="--", linewidth=0.5, alpha=0.4)
ax.set_axisbelow(True)

plt.ylabel("Mean duration (µs)")
plt.xlabel("Message Size")
plt.legend(title="Method", loc="upper left")

plt.savefig("out/k2u_2_actors_regress.pdf", bbox_inches="tight", pad_inches=0)