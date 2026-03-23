import numpy as np
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

from gen.setup import open_df, make_latency_latex_table

# Load data
k2k_data = open_df("k2k_2_act.csv").copy()
k2u_data = open_df("k2u_2_act.csv").copy()
u2u_data = open_df("u2u_2_act.csv").copy()

# Tag transport kind
k2k_data["kind"] = "k2k"
k2u_data["kind"] = "k2u"
u2u_data["kind"] = "u2u"

# Combine
data = pd.concat([k2k_data, k2u_data, u2u_data], ignore_index=True)

# Convert bytes -> KB
data["size_kb"] = data["size"] / 1000.0

# One series per (kind, op)
series_order = [
    ("k2k", "Move"),
    ("k2k", "Copy"),
    ("k2u", "Move"),
    ("k2u", "Copy"),
    ("u2u", "Move"),
    ("u2u", "Copy"),
]

palette = sns.color_palette("Pastel1", 9)
palette = [palette[i] for i in [0,1,2,3,4,6,7,8]]
palette = palette[:len(series_order)]

series_labels = {
    ("k2k", "Move"): "Kernel - Kernel / Page Table Move",
    ("k2k", "Copy"): "Kernel - Kernel / Message Copy",
    ("k2u", "Move"): "Kernel - User / Page Table Move",
    ("k2u", "Copy"): "Kernel - User / Message Copy",
    ("u2u", "Move"): "User - User / Page Table Move",
    ("u2u", "Copy"): "User - User / Message Copy",
}

print(make_latency_latex_table(data, series_labels))

# Pastel1 palette
color_map = {series: color for series, color in zip(series_order, palette)}

plt.figure(figsize=(12, 6.75))
ax = plt.gca()

# Clip everything above this value for display clarity
y_cap = 250
#ax.set_xlim(0, None)   # start at 0, auto max
ax.set_ylim(0, y_cap)

# Keep track of whether we already added the truncation note
has_clipped_points = False

# Plot points and regression line for each (kind, op)
for (kind, op) in series_order:
    subset = data[(data["kind"] == kind) & (data["op"] == op)].sort_values("size_kb")

    if subset.empty:
        continue

    x = subset["size_kb"].to_numpy()
    y = subset["mean_us"].to_numpy()
    color = color_map[(kind, op)]
    label = series_labels[(kind, op)]

    # Visible vs clipped points
    mask_visible = y <= y_cap
    mask_clipped = y > y_cap

    if np.any(mask_clipped):
        has_clipped_points = True

    # Scatter points shown normally
    ax.scatter(
        x[mask_visible],
        y[mask_visible],
        s=30,
        color=color,
        edgecolor="black",
        linewidth=0.8,
        zorder=3,
        label=label,
    )

    # Points above y_cap are marked at the top with triangles
    if np.any(mask_clipped):
        ax.scatter(
            x[mask_clipped],
            np.full(np.sum(mask_clipped), y_cap),
            marker="^",
            s=90,
            color=color,
            edgecolor="black",
            linewidth=0.8,
            zorder=4,
        )

    # True linear regression on the full, unclipped data
    m, b = np.polyfit(x, y, 1)

    x_line = np.linspace(x.min(), x.max(), 200)
    y_line = m * x_line + b

    # Only clip for display
    y_line_display = np.minimum(y_line, y_cap)

    ax.plot(
        x_line,
        y_line_display,
        color=color,
        linewidth=3.0,
        zorder=2,
    )

    # Put equation near the right end of the visible line
    y_label = min(m * x_line[-1] + b, y_cap - 5)

    if op == "Copy" and kind == "u2u":
        x_text = x[mask_visible][-1]
        y_text = y[mask_visible][-1] - 10

    elif op == "Move" and kind == "k2u":
        x_text = x_line[-1] - 15
        y_text = y[mask_visible][-1] - 23

    elif op == "Copy" and kind == "k2u":
        x_text = x_line[-1] - 15
        y_text = y[mask_visible][-1] + 10

    else:
        x_text = x_line[-1] - 15
        y_text = y_label + 10

    ax.text(
        x_text,
        y_text,
        f"y = {m:.2f}x + {b:.1f}",
        color=color,
        fontsize=12,
        va="center",
    )

# Axes
ax.set_xlabel("Message Size (KB)")
ax.set_ylabel("Mean duration (µs)")

# Legend
ax.legend(title="Method", loc="upper left")

# Grid
ax.yaxis.grid(True, linestyle="--", linewidth=0.5, alpha=0.4)
ax.set_axisbelow(True)

# Note that the plot is truncated if needed
if has_clipped_points:
    ax.text(
        0.99,
        0.98,
        f"Values above {y_cap} µs are clipped",
        transform=ax.transAxes,
        ha="right",
        va="top",
        fontsize=9,
        color="gray",
    )

plt.tight_layout()
plt.savefig("out/2_actor_comparison.pdf", bbox_inches="tight", pad_inches=0)