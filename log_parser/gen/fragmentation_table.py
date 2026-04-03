SIZES_BYTES = [1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000]


PAGE_4KIB = 4 * 1024
PAGE_2MIB = 2 * 1024 * 1024

def format_size_latex(num_bytes: int) -> str:
    if num_bytes < 1000:
        return f"\\SI{{{num_bytes}}}{{\\byte}}"
    elif num_bytes < 1024 * 1024:
        return f"\\SI{{{num_bytes / 1000:g}}}{{\\kilo\\byte}}"
    else:
        return f"\\SI{{{num_bytes / (1024 * 1024):g}}}{{\\mebi\\byte}}"

for size in SIZES_BYTES:
    byte_size = size

    frag_4kib = (
        f"${((PAGE_4KIB - byte_size) / PAGE_4KIB) * 100:.2f}\\%$"
        if byte_size <= PAGE_4KIB else "-"
    )
    frag_2mib = (
        f"${((PAGE_2MIB - byte_size) / PAGE_2MIB) * 100:.2f}\\%$"
        if byte_size <= PAGE_2MIB else "-"
    )

    size_str = format_size_latex(byte_size)
    print(f"{size_str} & {frag_4kib} & {frag_2mib} \\\\ \\hline")