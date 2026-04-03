# Rust Actor Kernel

Welcome to the rust actor kernel, a research focused OS which implements actors at
the kernel level in order to leverage the benefits such as memory safety. It explores
the options of memory copying vs mapping the memory through the virtual memory in form
of page tables.

_Note: The hardware deployment requires a Raspberry Pi 4B configured to Netboot via
TFTP. A TFTP server is required and must serve the directory `/srv/tftp/`_
