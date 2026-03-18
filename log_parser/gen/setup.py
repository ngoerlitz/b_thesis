import os.path

import matplotlib.axes
import pandas as pd
from matplotlib.patches import Rectangle


def _filename(filename: str) -> str:
    file = f"output/{filename}"
    if not os.path.isdir("output"):
        file = f"../output/{filename}"

    return file


def _format_size(n):
    if n < 1000:
        return f"{n} B"
    elif n < 1_000_000:
        return f"{n/1000:g} KB"
    elif n < 1_000_000_000:
        return f"{n/1_000_000:g} MB"
    else:
        return f"{n/1_000_000_000:g} GB"

def remove_outliers_iqr(df):
    grouped = df.groupby(["size", "op"])["duration_us"]

    q1 = grouped.transform(lambda s: s.quantile(0.25))
    q3 = grouped.transform(lambda s: s.quantile(0.75))
    iqr = q3 - q1

    lower = q1 - 1.5 * iqr
    upper = q3 + 1.5 * iqr

    return df[(df["duration_us"] >= lower) & (df["duration_us"] <= upper)]

def open_df(filename: str, remove_outliers: bool = False) -> pd.DataFrame:
    file = _filename(filename)
    df = pd.read_csv(file)

    df["op"] = df["op"].map({
        "MOV": "Move",
        "CPY": "Copy",
        "TST": "Test"
    })

    df["duration_us"] = df["duration_s"] * 1_000_000

    if remove_outliers:
        df = remove_outliers_iqr(df)

    df = df.groupby(["size", "op"], as_index=False).agg(
        mean_us=("duration_us", "mean"),
        std_us=("duration_us", "std"),
        count=("duration_us", "count")
    )

    df["std_us"] = df["std_us"].fillna(0)
    df["sem_us"] = df["std_us"] / df["count"] ** 0.5
    df["size_label"] = df["size"].apply(_format_size)

    size_order = sorted(df["size"].unique())

    df["size_label"] = pd.Categorical(
        df["size_label"],
        categories=[_format_size(s) for s in size_order],
        ordered=True
    )

    categories = ["Move", "Copy"]

    if "Test" in df["op"].values:
        categories.append("Test")

    df["op"] = pd.Categorical(df["op"], categories=categories, ordered=True)

    df = df.sort_values(["size_label", "op"])

    print(df)

    return df

def open_df_raw(filename: str) -> pd.DataFrame:
    file = _filename(filename)
    df = pd.read_csv(file)

    df["op"] = df["op"].map({
        "MOV": "Move",
        "CPY": "Copy"
    })

    df["duration_us"] = df["duration_s"] * 1_000_000

    return df

def bar_plot_err(ax: matplotlib.axes.Axes, data: pd.DataFrame):
    for container, op in zip(ax.containers, ["Move", "Copy"]):
        sub = data[data["op"] == op].sort_values("size")

        for bar, (_, row) in zip(container, sub.iterrows()):
            x = bar.get_x() + bar.get_width() / 2
            y = bar.get_height()

            ax.errorbar(
                x=x,
                y=y,
                yerr=row["std_us"],
                fmt="none",
                ecolor="black",
                capsize=3,
                elinewidth=1,
                capthick=1
            )

            ax.text(
                x,
                y + row["std_us"] + 0.5,
                f"{y:.1f}",
                ha="center",
                va="bottom",
                fontsize=9
            )