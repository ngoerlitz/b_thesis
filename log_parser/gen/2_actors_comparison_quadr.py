import numpy as np
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

from gen.setup import open_df, make_latency_latex_table

SIZES = [1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000]

# Load data
k2k_data = open_df("k2k_2_act.csv", SIZES).copy()
k2u_data = open_df("k2u_2_act.csv", SIZES).copy()
u2u_data = open_df("u2u_2_act.csv", SIZES).copy()

# Tag transport kind
k2k_data["kind"] = "k2k"
k2u_data["kind"] = "k2u"
u2u_data["kind"] = "u2u"

# Combine
data = pd.concat([k2k_data, k2u_data, u2u_data], ignore_index=True)

# Convert bytes -> kB
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

palette = sns.color_palette("Pastel1", 15)
palette = [palette[i] for i in [0, 1, 2, 3, 4, 6]]

series_labels = {
    ("k2k", "Move"): "Kernel to Kernel / Page Move",
    ("k2k", "Copy"): "Kernel to Kernel / Message Copy",
    ("k2u", "Move"): "Kernel to User / Page Move",
    ("k2u", "Copy"): "Kernel to User / Message Copy",
    ("u2u", "Move"): "User to User / Page Move",
    ("u2u", "Copy"): "User to User / Message Copy",
}

print(make_latency_latex_table(data, series_labels))

# Pastel1 palette
color_map = {series: color for series, color in zip(series_order, palette)}

# Minimum y-value for displaying fitted curves, per series.
# This only affects the displayed interpolation line, not the fit itself.
display_y_min = {
    ("u2u", "Copy"): 16,  # change this to whatever floor you want
}

plt.figure(figsize=(12, 6.75))
ax = plt.gca()

# Clip everything above this value for display clarity
y_cap = 200
ax.set_ylim(10, y_cap)
ax.set_xlim(-10, 260)

# Keep track of whether we already added the truncation note
has_clipped_points = False


def format_poly_eq(coeffs: np.ndarray) -> str:
    degree = len(coeffs) - 1
    if degree == 3:
        a, b, c, d = coeffs
        return f"y = {a:.2e}x³ + {b:.2e}x² + {c:.2f}x + {d:.1f}"
    if degree == 2:
        a, b, c = coeffs
        return f"y = {a:.2e}x² + {b:.2f}x + {c:.1f}"
    if degree == 1:
        m, b = coeffs
        return f"y = {m:.2f}x + {b:.1f}"
    return f"y = {coeffs[0]:.1f}"


# Plot points and cubic regression line for each (kind, op)
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

    # Cubic regression on the full, unclipped data
    # Fall back to a lower degree if there are too few data points.
    degree = 2
    coeffs = np.polyfit(x, y, degree)
    poly = np.poly1d(coeffs)

    x_line = np.linspace(x.min(), x.max(), 200)
    y_line = poly(x_line)

    # Display-only clipping
    y_floor = display_y_min.get((kind, op), None)
    if y_floor is not None:
        y_line_display = np.clip(y_line, y_floor, y_cap)
    else:
        y_line_display = np.minimum(y_line, y_cap)

    ax.plot(
        x_line,
        y_line_display,
        color=color,
        linewidth=3.0,
        zorder=2,
    )

    # Put equation near the right end of the visible line
    y_label_raw = float(poly(x_line[-1]))
    if y_floor is not None:
        y_label = min(max(y_label_raw, y_floor), y_cap - 5)
    else:
        y_label = min(y_label_raw, y_cap - 5)

    visible_indices = np.where(mask_visible)[0]
    if len(visible_indices) > 0:
        x_visible_last = x[visible_indices[-1]]
        y_visible_last = y[visible_indices[-1]]
    else:
        x_visible_last = x_line[-1]
        y_visible_last = y_cap - 10

    if op == "Copy" and kind == "u2u":
        x_text = x_visible_last
        y_text = max(y_visible_last - 10, display_y_min.get((kind, op), -np.inf))

    elif op == "Move" and kind == "k2u":
        x_text = x_line[-1] - 15
        y_text = y_visible_last - 23

    elif op == "Copy" and kind == "k2u":
        x_text = x_line[-1] - 15
        y_text = y_visible_last + 10

    else:
        x_text = x_line[-1] - 15
        y_text = y_label + 10

    # ax.text(
    #     x_text,
    #     y_text,
    #     format_poly_eq(coeffs),
    #     color=color,
    #     fontsize=12,
    #     va="center",
    # )

# Axes
ax.set_xlabel("Message Size (kB)")
ax.set_ylabel("Mean duration (µs)")

# Legend
ax.legend(title="Method", loc="upper left", markerscale=1.3, handleheight=1.2)

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
plt.savefig("out/2_actor_comparison_quadr.pdf", bbox_inches="tight", pad_inches=0)