import os.path
from unittest import case

import matplotlib.axes
import pandas as pd
from matplotlib.patches import Rectangle


def _filename(filename: str) -> str:
    if filename.startswith(".."):
        return filename

    file = f"output/{filename}"
    if not os.path.isdir("output"):
        file = f"../output/{filename}"

    return file

def get_label_legend(op):
    match op:
        case "Move":
            return "Page Table Move"
        case "Copy":
            return "Message Copy"

    return "ERROR"


def _format_size(n, force_decimals: bool):
    if n <= 128:
        return f"{n}"

    if n < 1000:
        return f"{n/1000:.1f} KB"

    if not force_decimals:
        return f"{n/1000:g} KB"
    else:
        return f"{n/1000:.1f} KB"

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

def open_df_matmul(filename: str, filter_sizes: list[int] | None = None, remove_outliers: bool = False, force_decimals: bool = False) -> pd.DataFrame:
    file = _filename(filename)
    df = pd.read_csv(file)

    if filter_sizes:
        df = df[df["size"].isin(filter_sizes)]

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
    df["size_label"] = df["size"]

    size_order = sorted(df["size"].unique())

    df["size_label"] = pd.Categorical(
        df["size_label"],
        categories=[s for s in size_order],
        ordered=True
    )

    df["op_label"] = df["op"].map(lambda v: get_label_legend(v))

    categories = ["Move", "Copy"]

    if "Test" in df["op"].values:
        categories.append("Test")

    df["op"] = pd.Categorical(df["op"], categories=categories, ordered=True)

    df = df.sort_values(["size_label", "op"])

    print(df)

    return df

def open_df(filename: str, filter_sizes: list[int] | None = None, remove_outliers: bool = False, force_decimals: bool = False) -> pd.DataFrame:
    file = _filename(filename)
    df = pd.read_csv(file)

    if filter_sizes:
        df = df[df["size"].isin(filter_sizes)]

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
    df["size_label"] = df["size"].apply(lambda x: _format_size(x, force_decimals))

    size_order = sorted(df["size"].unique())

    df["size_label"] = pd.Categorical(
        df["size_label"],
        categories=[_format_size(s, force_decimals) for s in size_order],
        ordered=True
    )

    df["op_label"] = df["op"].map(lambda v: get_label_legend(v))

    categories = ["Move", "Copy"]

    if "Test" in df["op"].values:
        categories.append("Test")

    df["op"] = pd.Categorical(df["op"], categories=categories, ordered=True)

    df = df.sort_values(["size_label", "op"])


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

def apply_basic_settings(ax: matplotlib.axes.Axes):
    ax.legend(title="Method", loc="upper left")
    ax.yaxis.grid(True, linestyle="--", linewidth=0.5, alpha=0.5)
    ax.set_axisbelow(True)

def bar_plot_err(ax: matplotlib.axes.Axes, data: pd.DataFrame, fmt: str = ".1f", error_bar_kws: dict | None = None):
    for container, op in zip(ax.containers, ["Move", "Copy"]):
        sub = data[data["op"] == op].sort_values("size")

        default_kws = {
            "elinewidth": 1,
            "capthick": 1,
            "capsize": 3,
            "ecolor": "black"
        }

        eb_kws = {**default_kws, **(error_bar_kws or {})}

        for bar, (_, row) in zip(container, sub.iterrows()):
            x = bar.get_x() + bar.get_width() / 2
            y = bar.get_height()

            ax.errorbar(
                x=x,
                y=y,
                yerr=row["std_us"],
                fmt="none",
                **eb_kws
            )

            ax.text(
                x,
                y + row["std_us"] + 0.5,
                format(y, fmt),
                ha="center",
                va="bottom",
                fontsize=9
            )

def line_plot_err(ax: matplotlib.axes.Axes, data: pd.DataFrame, offsets, fmt=".1f", error_bar_kws: dict | None = None, offset_text: bool = True, text_offset_mult: int = 2):
    for op in ["Move", "Copy"]:
        sub = data[data["op"] == op].sort_values("size")
        offset = offsets[op]

        default_kws = {
            "elinewidth": 1,
            "capthick": 1,
            "capsize": 3,
            "ecolor": "black"
        }

        eb_kws = {**default_kws, **(error_bar_kws or {})}

        for _, row in sub.iterrows():
            x = row["size"] + offset
            text_x = row["size"] + offset * text_offset_mult if offset_text else x
            y = row["mean_us"]

            ax.errorbar(
                x=x,
                y=y,
                yerr=row["std_us"],
                fmt="none",
                **eb_kws,
            )

            ax.text(
                text_x,
                y + row["std_us"] + 0.1,
                format(y, fmt),
                ha="center",
                va="bottom",
                fontsize=9
            )


def make_latency_latex_table(
        data: pd.DataFrame,
        series_labels: dict,
        value_col: str = "mean_us",
        size_col: str = "size",
        kind_col: str = "kind",
        op_col: str = "op",
        float_fmt: str = ".2f",
        missing: str = "--",
        use_booktabs: bool = True,
) -> str:
    """
    Build a LaTeX table for the combined k2k / k2u / u2u dataframe.

    Layout:
      - Rows: message size, with two sub-rows per size (Move / Copy)
      - Columns: transport kinds (Kernel-Kernel, Kernel-User, User-User)
      - Cells: mean_us values

    This uses:
      - \\multirow for message size
      - \\multicolumn for grouped column headers

    Expected input:
      data: combined dataframe containing columns like:
            ["size", "kind", "op", "mean_us"]
      series_labels: mapping like:
            ("k2k", "Move") -> "Kernel - Kernel / Page Table Move"
            ("k2k", "Copy") -> "Kernel - Kernel / Message Copy"
            ...

    Required LaTeX packages:
      \\usepackage{booktabs}
      \\usepackage{multirow}
    """

    df = data.copy()

    # Preserve logical order from series_labels
    series_order = list(series_labels.keys())
    kind_order = []
    op_order = []

    for kind, op in series_order:
        if kind not in kind_order:
            kind_order.append(kind)
        if op not in op_order:
            op_order.append(op)

    # Derive pretty kind labels from series_labels, e.g.
    # ("k2k", "Move") -> "Kernel - Kernel / Page Table Move"
    # kind label becomes "Kernel - Kernel"
    kind_display = {}
    for (kind, op), label in series_labels.items():
        transport, _ = label.split(" / ", 1)
        kind_display[kind] = transport

    # Derive pretty op labels from series_labels, e.g.
    # "Page Table Move" / "Message Copy"
    op_display = {}
    for (kind, op), label in series_labels.items():
        _, method = label.split(" / ", 1)
        op_display[op] = method

    # Pivot so that index = (size, op), columns = kind, values = mean_us
    pivot = (
        df.pivot_table(
            index=[size_col, op_col],
            columns=kind_col,
            values=value_col,
            aggfunc="mean",
        )
        .sort_index()
    )

    sizes = sorted(df[size_col].dropna().unique())

    def fmt_size(n: int) -> str:
        if n % 1000 == 0:
            return f"{int(n/1000)} KB"
        return f"{n} B"

    def fmt_val(x):
        if pd.isna(x):
            return missing
        return format(float(x), float_fmt)

    lines = []

    # Column specification: 2 left columns + one numeric column per kind
    col_spec = "ll" + "r" * len(kind_order)

    lines.append("\\begin{table}[htbp]")
    lines.append("\\centering")
    lines.append("\\small")
    lines.append("\\setlength{\\tabcolsep}{6pt}")

    if use_booktabs:
        lines.append(f"\\begin{{tabular}}{{{col_spec}}}")
        lines.append("\\toprule")
        lines.append(
            " &  & "
            + " & ".join(
                f"\\multicolumn{{1}}{{c}}{{{kind_display[k]}}}" for k in kind_order
            )
            + " \\\\"
        )
        lines.append("\\cmidrule(lr){3-" + str(2 + len(kind_order)) + "}")
        lines.append(
            "Message Size & Type & "
            + " & ".join(kind_display[k] for k in kind_order)
            + " \\\\"
        )
        lines.append("\\midrule")
    else:
        lines.append(f"\\begin{{tabular}}{{{col_spec}}}")
        lines.append("\\hline")
        lines.append(
            " &  & "
            + " & ".join(
                f"\\multicolumn{{1}}{{c}}{{{kind_display[k]}}}" for k in kind_order
            )
            + " \\\\"
        )
        lines.append("\\cline{3-" + str(2 + len(kind_order)) + "}")
        lines.append(
            "Message Size & Type & "
            + " & ".join(kind_display[k] for k in kind_order)
            + " \\\\"
        )
        lines.append("\\hline")

    # One multirow per size, with one row for each op
    for size in sizes:
        for i, op in enumerate(op_order):
            row_cells = []

            # Leftmost size cell
            if i == 0:
                row_cells.append(f"\\multirow{{{len(op_order)}}}{{*}}{{{fmt_size(size)}}}")
            else:
                row_cells.append("")

            # Type / method row label
            row_cells.append(op_display.get(op, op))

            # Data cells across kinds
            for kind in kind_order:
                val = pivot.loc[(size, op), kind] if (size, op) in pivot.index and kind in pivot.columns else pd.NA
                row_cells.append(fmt_val(val))

            lines.append(" & ".join(row_cells) + " \\\\")

        # separator after each size block
        if use_booktabs:
            lines.append("\\addlinespace[2pt]")
        else:
            lines.append("\\hline")

    if use_booktabs:
        # remove trailing addlinespace if present
        if lines[-1] == "\\addlinespace[2pt]":
            lines.pop()
        lines.append("\\bottomrule")
    else:
        if lines[-1] == "\\hline":
            pass
        else:
            lines.append("\\hline")

    lines.append("\\end{tabular}")
    lines.append("\\caption{Mean message transfer duration (\\,$\\mu$s).}")
    lines.append("\\label{tab:message-transfer-latency}")
    lines.append("\\end{table}")

    return "\n".join(lines)