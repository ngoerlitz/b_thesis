import matplotlib.pyplot as plt
import pandas as pd
import seaborn

from gen.setup import open_df, open_df_raw, bar_plot_err, apply_basic_settings

SIZES = [1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000]

k2k_data = open_df("input_k2k_2_actors.csv", SIZES)
k2u_data = open_df("input_k2u_2_actors.csv", SIZES)
u2u_data = open_df("input_u2u_2_actors.csv", SIZES)

k2k_data["type"] = "k2k"
k2u_data["type"] = "k2u"
u2u_data["type"] = "u2u"

print(pd.concat([k2k_data, k2u_data, u2u_data]))

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

apply_basic_settings(ax)
bar_plot_err(ax, data, error_bar_kws={
    "capthick": 0.8,
    "elinewidth": 0.8
})

print(data)

plt.ylabel("Mean duration (µs)")
plt.xlabel("Message Size")
plt.savefig("out/test.pdf", bbox_inches="tight", pad_inches=0)