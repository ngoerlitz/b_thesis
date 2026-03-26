import numpy as np

from gen.setup import open_df
from gen.tex.bar_table import gen_table

SIZES = [1,2,4,8,16,32,64,128]

data = open_df("../../output/matmul.csv", SIZES)
gen_table(data, output_file="../out/matmul_table.tex", col_width="2.8cm" ,header=fr"\textbf{{Matrix Size}} & \textbf{{Copy}} & \textbf{{Move}} & \textbf{{$\Delta$}} & \textbf{{Ratio}} \\ \toprule")

