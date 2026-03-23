#!/usr/bin/env python3
import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

# python3 test_runner.py --config test_cfg.json && ./build.sh rpi --release

AUTO_TEST_BLOCK_PATTERN = re.compile(
    r"(?P<begin>^[ \t]*// AUTO_TEST_BEGIN[ \t]*\n)"
    r"(?P<line>.*?)"
    r"(?P<end>\n[ \t]*// AUTO_TEST_END[ \t]*$)",
    re.DOTALL | re.MULTILINE,
)


def patch_rust_file(rust_file: Path, size: int, test_path: str) -> None:
    content = rust_file.read_text(encoding="utf-8")

    replacement_line = f"run_test!({size}, {test_path});"
    new_content, count = AUTO_TEST_BLOCK_PATTERN.subn(
        rf"\1{replacement_line}\3", content, count=1
    )

    if count != 1:
        raise RuntimeError(
            f"Could not find exactly one AUTO_TEST block in {rust_file}.\n"
            "Expected:\n"
            "// AUTO_TEST_BEGIN\n"
            "run_test!(..., ...);\n"
            "// AUTO_TEST_END"
        )

    rust_file.write_text(new_content, encoding="utf-8")


def load_config(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def normalize_config(config: dict) -> tuple[Path, list[int], list[tuple[str, str]]]:
    rust_file = config.get("rust_file")
    if not rust_file:
        raise ValueError("Missing 'rust_file' in config")

    sizes = config.get("sizes")
    if not isinstance(sizes, list) or not sizes:
        raise ValueError("'sizes' must be a non-empty array")

    tests = config.get("tests")
    if not isinstance(tests, list) or not tests:
        raise ValueError("'tests' must be a non-empty array")

    normalized_sizes = []
    for s in sizes:
        normalized_sizes.append(int(s))

    normalized_tests = []
    for entry in tests:
        if not isinstance(entry, dict):
            raise ValueError("Each entry in 'tests' must be an object")
        name = entry.get("name")
        path = entry.get("path")
        if not name or not path:
            raise ValueError("Each test must contain 'name' and 'path'")
        normalized_tests.append((str(name), str(path)))

    return Path(rust_file), normalized_sizes, normalized_tests


def build_plan(sizes: list[int], tests: list[tuple[str, str]]) -> list[tuple[str, str, int]]:
    plan = []
    for size in sizes:
        for test_name, test_path in tests:
            plan.append((test_name, test_path, size))
    return plan


def config_fingerprint(rust_file: Path, sizes: list[int], tests: list[tuple[str, str]]) -> str:
    payload = {
        "rust_file": str(rust_file),
        "sizes": sizes,
        "tests": [{"name": n, "path": p} for n, p in tests],
    }
    raw = json.dumps(payload, sort_keys=True).encode("utf-8")
    return hashlib.sha256(raw).hexdigest()


def get_state_file(config_path: Path) -> Path:
    return config_path.with_suffix(config_path.suffix + ".state.json")


def load_state(state_file: Path) -> dict:
    if not state_file.exists():
        return {}
    with state_file.open("r", encoding="utf-8") as f:
        return json.load(f)


def save_state(state_file: Path, state: dict) -> None:
    with state_file.open("w", encoding="utf-8") as f:
        json.dump(state, f, indent=2)


def print_plan(plan: list[tuple[str, str, int]]) -> None:
    print("Planned runs:")
    for i, (test_name, test_path, size) in enumerate(plan, start=1):
        print(f"{i:3d}. size={size:<8} test={test_name:<16} path={test_path}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Patch a Rust AUTO_TEST block from a JSON config. "
            "Each repeated call with the same parameters advances to the next run."
        )
    )
    parser.add_argument("--config", required=True, help="Path to JSON config file")
    parser.add_argument(
        "--print-plan",
        action="store_true",
        help="Print the full execution plan",
    )
    parser.add_argument(
        "--reset",
        action="store_true",
        help="Reset progress back to the first run",
    )
    parser.add_argument(
        "--show-current",
        action="store_true",
        help="Show the next planned run without modifying the Rust file",
    )

    args = parser.parse_args()

    try:
        config_path = Path(args.config)
        config = load_config(config_path)
        rust_file, sizes, tests = normalize_config(config)
        plan = build_plan(sizes, tests)
        fingerprint = config_fingerprint(rust_file, sizes, tests)

        state_file = get_state_file(config_path)
        state = load_state(state_file)

        if args.reset:
            state = {
                "fingerprint": fingerprint,
                "next_index": 0,
            }
            save_state(state_file, state)
            print(f"Reset state: {state_file}")

        if args.print_plan:
            print_plan(plan)

        if not rust_file.exists():
            raise FileNotFoundError(f"Rust file not found: {rust_file}")

        stored_fingerprint = state.get("fingerprint")
        next_index = int(state.get("next_index", 0))

        if stored_fingerprint != fingerprint:
            next_index = 0
            state = {
                "fingerprint": fingerprint,
                "next_index": 0,
            }

        if next_index >= len(plan):
            print("All runs are complete.")
            print(f"Reset with: python3 {Path(__file__).name} --config {config_path} --reset")
            return 0

        test_name, test_path, size = plan[next_index]

        if args.show_current:
            print(f"Next run: {next_index + 1}/{len(plan)}")
            print(f"Size : {size}")
            print(f"Test : {test_name}")
            print(f"Path : {test_path}")
            print(f"Line : run_test!({size}, {test_path});")
            return 0

        patch_rust_file(rust_file, size, test_path)

        state = {
            "fingerprint": fingerprint,
            "next_index": next_index + 1,
        }
        save_state(state_file, state)

        print(f"Updated {rust_file}")
        print(f"Applied run {next_index + 1}/{len(plan)}")
        print(f"Size : {size}")
        print(f"Test : {test_name}")
        print(f"Path : {test_path}")
        print(f"Line : run_test!({size}, {test_path});")

        if next_index + 1 < len(plan):
            next_test_name, next_test_path, next_size = plan[next_index + 1]
            print()
            print("Next call will apply:")
            print(f"Size : {next_size}")
            print(f"Test : {next_test_name}")
            print(f"Path : {next_test_path}")
        else:
            print()
            print("This was the final run in the plan.")

        return 0

    except Exception as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())