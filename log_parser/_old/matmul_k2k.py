import matplotlib.pyplot as plt
import seaborn

from gen.setup import open_df, open_df_raw, bar_plot_err, apply_basic_settings, open_df_matmul

SIZES = [1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000]

data = open_df_matmul("input_matmul_k2k.csv")

print(data)

plt.figure(figsize=(12, 6.75))

ax = seaborn.barplot(
    data=data,
    x="size_label",
    y="mean_us",
    hue="op_label",
    palette="Pastel1",
    errorbar=None,
    edgecolor="black",
    linewidth=0.5
)

apply_basic_settings(ax)
bar_plot_err(ax, data, fmt=".0f", error_bar_kws={
    "capthick": 0.8,
    "elinewidth": 0.8
})

plt.ylabel("Mean duration (µs)")
plt.xlabel("Matrix size (n x n)")
plt.savefig("out/input_matmul_k2k.pdf", bbox_inches="tight", pad_inches=0)