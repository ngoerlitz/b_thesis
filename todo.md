- [] Fix kernel panic bug [1 Day! - 07.02.2026]

- [] Write frame allocator in 1GB - 4GB range [2 Days, 09.02 - 10.02]
- [] When actor is spawned, reserve one frame and assign it to `OUTBOX_ADDR` (static), through MMU
- [] Store information on page tables for actor (inside handle method) 
- [] EL1 -> EL0 switch, page table must be mapped for specific actor → map Inbox / Outbox to fixed addrs. [2 Days]

=== [5 Weeks]

- [] Benchmarks [1 Week]

= [6 Weeks / 11 Weeks (Mitte März)]

- Writeup [4 Weeks; Mitte März - 20.04]



Channel anpassen


Wrap types in core for kernel (not just "M", but Enum)
schau das der enum nicht zu groß wird. 