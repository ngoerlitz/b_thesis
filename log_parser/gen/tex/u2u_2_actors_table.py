import numpy as np

from gen.setup import open_df
from gen.tex.bar_table import gen_table

SIZES = [1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000]

data = open_df("u2u_2_act.csv", SIZES)
gen_table(data, label="u2u_msg", caption="Results of single message U2U experiment with difference and ratio",  output_file="./out/u2u_2_actors_table.tex")

