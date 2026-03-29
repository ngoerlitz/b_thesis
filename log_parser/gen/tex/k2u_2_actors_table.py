from gen.tex.bar_table import gen_table
from gen.setup import open_df

SIZES = [1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000]

data = open_df("../../output/k2u_2_act.csv", SIZES)
gen_table(data, label="k2u_msg", caption="Results of single K2U message experiment with difference and ratio", output_file="../out/k2u_2_actors_table.tex")
