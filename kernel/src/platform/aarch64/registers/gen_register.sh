#!/bin/sh
# Usage: ./gen.sh <reg_name>

set -eu

if [ $# -ne 1 ]; then
  echo "Usage: $0 <reg_name>" >&2
  exit 1
fi

reg_name=$(printf '%s' "$1" | tr '[:lower:]' '[:upper:]')
file_name=$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')

echo $reg_name ">>" $file_name

cat > "$file_name.rs" <<EOF
#[allow(non_snake_case)]
pub mod ${reg_name} {
    use crate::aarch64_read_write_system_reg;
    use core::arch::asm;

    pub struct Register {}

    impl Register {
        aarch64_read_write_system_reg!(u64, "${reg_name}");
    }
}

pub static ${reg_name}: ${reg_name}::Register = ${reg_name}::Register {};
EOF

sed -i "1i\pub mod $file_name;" mod.rs