import os.path

from input_parser import InputParser

TIMER_FRQ = 54000000

def input_to_csv(path: str):
    if not os.path.isdir(path):
        raise Exception("Input directory does not exist")

    if not os.path.isdir("output"):
        os.mkdir("output")

    for file in os.listdir(path):
        if file.endswith(".txt"):
            filename = file.replace(".txt", "")
            parser = InputParser(f"{path}/{file}", f"output/{filename}.csv", TIMER_FRQ)
            parser.parse()
            parser.write_csv()

input_to_csv("input")