- [] Kernel memory manager (stack, pages, etc.) [1 Day]

- [] Multi-Core Support: [1 Week]
    - [] MMU support for multiple cores -> multiple PTEs for each core with specific addresses for Actors -->
      RootEnvironment {actors: HashMap<...>}? [6 Days]

-- Latest 25. Jan

- [] Message Passing via Page-Table-Remapping [4 Weeks + ??]
    - [] Reserving heap memory in [1GB - 4GB] range. Initially 1 page, syscall for more if required [2 Days]
    - [] MMU mapping for actor with addr. of inbox/outbox [7 Days]
    - [] Send message via syscall (remapping in kernel) [7 Days]
    - [] Receive messages from inbox [2 Days]

=== [5 Weeks]

- [] Benchmarks [1 Week]

= [6 Weeks / 11 Weeks (Mitte März)]

- Writeup [4 Weeks; Mitte März - 20.04]