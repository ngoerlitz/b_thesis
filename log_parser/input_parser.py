import csv
import os.path
import re
from dataclasses import dataclass


@dataclass
class InputData:
    operation: str
    start_tick: int
    end_tick: int
    duration_s: float

SIZE_PATTERN = re.compile(r"^SIZE:\s(\d+)$")
DATA_PATTERN = pattern = re.compile(r"^\s*(?:User:\s*)?[User:\s]?\[(MOV|CPY)]\s*(->|<-)\s*(\d+)\s*$")

class InputParser:
    def __init__(self, path: str, out: str, timer_frq: int) -> None:
        self.path = path
        self.out = out
        self.timer_frq = timer_frq
        self.results: dict[int, list[InputData]] = {}

        if not os.path.exists(self.path):
            raise Exception(f"File {self.path} does not exist")


    def _extract_sizes(self, line_num: int, match: re.Match[str] | None) -> int:
        if match is None:
            raise Exception(f"Failed to extract SIZE in line {line_num} | File: {self.path}")

        return int(match.group(1))


    def _extract_data(
            self,
            cur_size: int | None,
            line_num: int,
            pending: dict[str, int | None],
            match: re.Match[str] | None
    ) -> None:
        if cur_size is None:
            raise ValueError(f"Encountered data before SIZE on line {line_num} | File: {self.path}")

        if match is None:
            raise ValueError(f"Invalid data match on line {line_num} | File: {self.path}")

        op = match.group(1)
        direction = match.group(2)
        tick = int(match.group(3))

        prev = pending[op]

        if direction == "->":
            if prev is not None:
                raise ValueError(
                    f"Encountered repeated '->' for {op} without matching '<-' on line {line_num} | File: {self.path}"
                )
            pending[op] = tick
            return

        # direction == "<-"
        if prev is None:
            raise ValueError(
                f"Encountered '<-' for {op} without previous '->' on line {line_num} | File: {self.path}"
            )

        delta = tick - prev
        if delta < 0:
            raise ValueError(
                f"Negative tick delta for {op} on line {line_num}: start={prev}, end={tick} | File: {self.path}"
            )

        time_us = delta / self.timer_frq

        self.results[cur_size].append(InputData(op, prev, tick, time_us))

        pending[op] = None


    def parse(self) -> None:
        with open(self.path, "r", encoding="utf-8") as infile:
            cur_size: int | None = None
            pending: dict[str, int | None] = {"MOV": None, "CPY": None}

            for i, line in enumerate(infile, start=1):
                m = SIZE_PATTERN.match(line)
                d = DATA_PATTERN.match(line)

                if line.startswith("#"):
                    continue

                if m:
                    cur_size = self._extract_sizes(i, m)
                    self.results[cur_size] = []

                    pending = {"MOV": None, "CPY": None}
                    continue

                if d:
                    self._extract_data(cur_size, i, pending, d)

            dangling = [op for op, value in pending.items() if value is not None]
            if dangling:
                raise ValueError(
                    f"Unmatched '->' remaining at end of file for: {', '.join(dangling)} | File: {self.path}"
                )


    def write_csv(self):
        with open(self.out, "w", newline="", encoding="utf-8") as outfile:
            writer = csv.writer(outfile)

            writer.writerow([
                "id",
                "size",
                "op",
                "start_tick",
                "end_tick",
                "duration_s"
            ])

            row_id = 0

            for size in sorted(self.results.keys()):
                for entry in self.results[size]:
                    writer.writerow([
                        row_id,
                        size,
                        entry.operation,
                        entry.start_tick,
                        entry.end_tick,
                        entry.duration_s
                    ])

                    row_id += 1

            print(self.results)