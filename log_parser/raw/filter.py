#!/usr/bin/env python3
from pathlib import Path

KEYWORDS = ("SIZE:", "[CPY]", "[MOV]")


def should_keep(line: str) -> bool:
    return any(k in line for k in KEYWORDS)


def process_file(path: Path):
    output_path = path.with_suffix(path.suffix + ".filtered")

    last_size_line = None

    with path.open("r", encoding="utf-8", errors="ignore") as infile, \
            output_path.open("w", encoding="utf-8") as outfile:

        for raw_line in infile:
            line = raw_line.rstrip("\n")

            if not should_keep(line):
                continue

            # Handle SIZE lines
            if line.startswith("SIZE:"):
                if line == last_size_line:
                    # skip duplicate SIZE
                    continue

                # new SIZE block → add spacing
                outfile.write("\n" + line + "\n")
                last_size_line = line
                continue

            # Non-SIZE lines
            outfile.write(line + "\n")

    print(f"Processed: {path.name} -> {output_path.name}")


def main():
    log_files = list(Path(".").glob("*.log"))

    if not log_files:
        print("No .log files found.")
        return

    for log_file in log_files:
        process_file(log_file)


if __name__ == "__main__":
    main()