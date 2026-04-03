#!/usr/bin/env python3
"""
Fix misordered trace lines by sorting event lines inside each SIZE block.

Sorting rules inside each SIZE block:
1. All [CPY] lines come before all [MOV] lines.
2. Within the same prefix, lines are sorted by timestamp ascending.
3. Stable ordering is preserved for identical timestamps.

Example:
    SIZE: 1
    [MOV] -> 4
    [CPY] -> 1
    [CPY] <- 2
    [MOV] <- 8

becomes:
    SIZE: 1
    [CPY] -> 1
    [CPY] <- 2
    [MOV] -> 4
    [MOV] <- 8
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


EVENT_RE = re.compile(
    r"^(?P<user>User:\s+)?(?P<prefix>\[[^\]]+\])\s+(?P<arrow>->|<-)\s+(?P<ts>\d+)\s*$"
)
SIZE_RE = re.compile(r"^SIZE:\s*(?P<size>\S+)\s*$")


@dataclass(frozen=True)
class Event:
    original_index: int
    prefix: str
    arrow: str
    timestamp: int
    raw_line: str


@dataclass
class Block:
    header: str
    events: list[Event]


def parse_blocks(lines: list[str]) -> tuple[list[str], list[Block]]:
    """
    Parse the file into:
      - preamble lines before the first SIZE block
      - SIZE blocks, each containing event lines

    Blank lines are ignored between sections.
    """
    preamble: list[str] = []
    blocks: list[Block] = []

    current_block: Block | None = None
    event_index = 0

    for lineno, line in enumerate(lines, start=1):
        stripped = line.rstrip("\n")

        if not stripped:
            continue

        size_match = SIZE_RE.match(stripped)
        if size_match:
            current_block = Block(header=stripped, events=[])
            blocks.append(current_block)
            continue

        event_match = EVENT_RE.match(stripped)
        if event_match:
            if current_block is None:
                raise ValueError(
                    f"Encountered event line before first SIZE block on line {lineno}: {stripped!r}"
                )

            current_block.events.append(
                Event(
                    original_index=event_index,
                    prefix=event_match.group("prefix"),
                    arrow=event_match.group("arrow"),
                    timestamp=int(event_match.group("ts")),
                    raw_line=stripped,
                )
            )
            event_index += 1
            continue

        if current_block is None:
            preamble.append(stripped)
        else:
            raise ValueError(
                f"Unexpected non-event line inside SIZE block on line {lineno}: {stripped!r}"
            )

    return preamble, blocks


def prefix_rank(prefix: str) -> tuple[int, str]:
    """
    Enforce [CPY] before [MOV]. Any other prefixes come afterwards, alphabetically.
    """
    if prefix == "[CPY]":
        return (0, prefix)
    if prefix == "[MOV]":
        return (1, prefix)
    return (2, prefix)


def fix_block(events: list[Event]) -> list[str]:
    """
    Sort block events with [CPY] before [MOV], then by timestamp ascending.

    Stable sort preserves original order for identical keys.
    """
    sorted_events = sorted(
        events,
        key=lambda e: (prefix_rank(e.prefix), e.timestamp, e.original_index),
    )
    return [e.raw_line for e in sorted_events]


def validate_block(header: str, event_lines: list[str]) -> None:
    """
    Validate that no '<-' appears before a matching '->' for the same prefix
    within a single SIZE block.
    """
    open_counts: dict[str, int] = {}

    for idx, line in enumerate(event_lines, start=1):
        match = EVENT_RE.match(line)
        if not match:
            continue

        prefix = match.group("prefix")
        arrow = match.group("arrow")

        open_counts.setdefault(prefix, 0)

        if arrow == "->":
            open_counts[prefix] += 1
        else:
            if open_counts[prefix] == 0:
                raise ValueError(
                    f"Encountered '<-' for {prefix} without previous '->' in block {header!r} "
                    f"at event line {idx}"
                )
            open_counts[prefix] -= 1


def process_text(text: str) -> str:
    lines = text.splitlines()
    preamble, blocks = parse_blocks(lines)

    out_lines: list[str] = []
    if preamble:
        out_lines.extend(preamble)

    for block in blocks:
        fixed_events = fix_block(block.events)
        validate_block(block.header, fixed_events)

        if out_lines:
            out_lines.append("")
        out_lines.append(block.header)
        out_lines.extend(fixed_events)

    return "\n".join(out_lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Fix misordered trace lines within each SIZE block."
    )
    parser.add_argument("input", type=Path, help="Input text file")
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        help="Output file (defaults to stdout)",
    )
    args = parser.parse_args()

    try:
        text = args.input.read_text(encoding="utf-8")
        fixed = process_text(text)
    except Exception as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1

    if args.output:
        args.output.write_text(fixed, encoding="utf-8")
    else:
        sys.stdout.write(fixed)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())