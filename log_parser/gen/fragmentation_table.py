PAGE_4KIB = 4 * 1024
PAGE_2MIB = 2 * 1024 * 1024

def format_size_latex(num_bytes: int) -> str:
    if num_bytes < 1024:
        return f"\\SI{{{num_bytes}}}{{\\byte}}"
    elif num_bytes < 1024 * 1024:
        return f"\\SI{{{num_bytes / 1024:g}}}{{\\kibi\\byte}}"
    else:
        return f"\\SI{{{num_bytes / (1024 * 1024):g}}}{{\\mebi\\byte}}"

for exp in range(3, 22, 1):  # 2^3 = 8 B, ..., 2^21 = 2 MiB
    byte_size = 1 << exp

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