from gen.setup import open_df
from gen.tex.bar_table import gen_table

SIZES = [1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000]

data = open_df("../../output/k2k_2_act.csv", SIZES)
gen_table(data, label="test", output_file="../out/k2k_2_actors_table.tex")

