# Generate a latex table that is to be used in conjunction with the corresponding bar
# graphs like k2k_2_actors, or k2u_2_actors or u2u_2_actors
import numpy as np
import pandas as pd


def build_table(data: str, caption: str | None, label: str | None, header: str | None, col_width: str) -> str:
    if header is None:
        header = fr"\textbf{{Message Size}} & \textbf{{Copy}} & \textbf{{Move}} & \textbf{{$\Delta$}} & \textbf{{Ratio}} \\ \toprule"

    return fr"""
\begin{{table}}[H]
    \centering
    \small
    \setlength{{\tabcolsep}}{{4pt}}
    \renewcommand{{\arraystretch}}{{1.3}}
    \begin{{tabular}}{{l C{{{col_width}}} C{{{col_width}}} C{{{col_width}}} C{{{col_width}}}}}
        \hline
        & \multicolumn{{3}}{{c}}{{\textbf{{Mean Duration (\si{{\us}})}}}} & \\
        {header}
        {data}
    \end{{tabular}}
    \caption{{{caption}}}
    \label{{tab:{label }}}
\end{{table}}
"""

def gen_table(data: pd.DataFrame, output_file: str | None, header: str | None = None, caption: str = r"\color{red}{Temp Caption}", label: str = "temp_label", col_width: str = "2.5cm") -> str:
    tabledata = ""

    for i in range(0, len(data), 2):
        mov = data.iloc[i]
        cpy = data.iloc[i+1]

        mov_str = fr"{mov['mean_us']:.2f} $\pm$ {mov['std_us']:.2f}"
        cpy_str = fr"{cpy['mean_us']:.2f} $\pm$ {cpy['std_us']:.2f}"

        delta = cpy["mean_us"] - mov["mean_us"]
        delta_std = np.sqrt(
            cpy["std_us"]**2 + mov["std_us"]**2
        )

        rel = cpy["mean_us"] / mov["mean_us"]
        rel_err = np.sqrt(
            (cpy["std_us"] / cpy["mean_us"])**2 +
            (mov["std_us"] / mov["mean_us"])**2
        ) * rel

        tabledata += fr"""
    {mov["size_label"]} & {cpy_str} & {mov_str} & {delta:.2f} $\pm$ {delta_std:.2f} & {rel:.2f} $\pm$ {rel_err:.2f} \\ \hline"""

    data = build_table(tabledata, caption, label, header, col_width)

    if output_file is not None:
        with open(output_file, "w") as f:
            f.write(data)

    return data
