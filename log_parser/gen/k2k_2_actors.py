import matplotlib.pyplot as plt
import seaborn

from gen.setup import open_df, open_df_raw, bar_plot_err

data = open_df("input_k2k_2_actors.csv")

plt.figure(figsize=(12, 6.75))

ax = seaborn.barplot(
    data=data,
    x="size_label",
    y="mean_us",
    hue="op",
    palette="Pastel1",
    errorbar=None,
    edgecolor="black",
    linewidth=0.5
)

ax.legend(title="Method")

bar_plot_err(ax, data)

plt.ylabel("Mean duration (µs)")
plt.xlabel("Message Size")
plt.savefig("output/input_k2k_2_actors.pdf", bbox_inches="tight", pad_inches=0)