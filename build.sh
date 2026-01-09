#!/bin/bash
set -euo pipefail

MODE=""
PROFILE="debug"   # debug|release
GDB=0
LOG_DEBUG=0
EXTRA_QEMU_ARGS=()

usage() {
  cat <<'USAGE'
Usage:
  build.sh <qemu|rpi> [--release] [--gdb] [--log-debug] [-- <extra qemu args>]

Modes:
  qemu       Build QEMU feature set, create kernel8.img, run QEMU
  rpi        Build hardware feature set, create kernel8.img, copy to /srv/tftp/my.img

Options:
  --release    Build with cargo --release (otherwise debug)
  --gdb        (qemu only) Start QEMU with -s -S for gdb attach
  --log-debug  Enable cargo feature flag 'log_debug'
  -h, --help   Show this help

Notes:
  - Extra arguments after "--" are forwarded to qemu-system-aarch64.
  - Adjust BIN_NAME / TARGET_TRIPLE / TFTP_DEST as needed.
USAGE
}

# Parse positional mode
if [[ $# -lt 1 ]]; then
  usage
  exit 2
fi
MODE="$1"; shift

# Parse flags
while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      PROFILE="release"; shift ;;
    --gdb)
      GDB=1; shift ;;
    --log-debug)
      LOG_DEBUG=1; shift ;;
    --help|-h)
      usage; exit 0 ;;
    --)
      shift
      EXTRA_QEMU_ARGS+=("$@")
      break ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

TARGET_TRIPLE="aarch64-unknown-none-softfloat"
BIN_NAME="kernel-playground"
OUT_DIR="./target/out"
IMG_PATH="$OUT_DIR/kernel8.img"
TFTP_DEST="/srv/tftp/my.img"

cargo_profile_flags=()
if [[ "$PROFILE" == "release" ]]; then
  cargo_profile_flags+=("--release")
fi

artifact_path="target/${TARGET_TRIPLE}/${PROFILE}/${BIN_NAME}"

build_and_objcopy() {
  local base_feature="$1"

  mkdir -p "$OUT_DIR"

  # Compose feature list: base mode feature + optional log_debug
  local features="$base_feature"
  if [[ $LOG_DEBUG -eq 1 ]]; then
    features+=",log_debug"
  fi

  cargo build --target "$TARGET_TRIPLE" "${cargo_profile_flags[@]}" --features "$features"

  if [[ ! -f "$artifact_path" ]]; then
    echo "Expected artifact not found: $artifact_path" >&2
    exit 1
  fi

  aarch64-none-elf-objcopy -O binary "$artifact_path" "$IMG_PATH"
}

case "$MODE" in
  qemu)
    build_and_objcopy "qemu"

    # Fresh log
    : > "$OUT_DIR/qemu.log"

    qemu_args=(
      -machine raspi4b
      -kernel "$IMG_PATH"
      -nographic
      -m 2048
      -d mmu
      -D "$OUT_DIR/qemu.log"
    )

    if [[ $GDB -eq 1 ]]; then
      qemu_args+=( -s -S )
    fi

    qemu_args+=("${EXTRA_QEMU_ARGS[@]}")

    exec qemu-system-aarch64 "${qemu_args[@]}"
    ;;

  rpi)
    if [[ $GDB -eq 1 ]]; then
      echo "--gdb is only supported with mode 'qemu'" >&2
      exit 2
    fi

    build_and_objcopy "hardware"

    sudo cp "$IMG_PATH" "$TFTP_DEST"
    echo "Copied $IMG_PATH -> $TFTP_DEST"
    ;;

  *)
    echo "Unknown mode: '$MODE'" >&2
    usage
    exit 2
    ;;
esac
