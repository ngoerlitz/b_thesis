- [X] EL1 -> EL0 switch in UserActor [0.5 Day] 
- [X] Message Passing via Copying in UserActor --> basically done through SVC, once [0;1] complete. [1 Day] X

- [] UserActor must be able to send messages to other actors too. This requires some sort of mapping from <ActorId <-> Channel> [1 Day -- High Priority!!]
     this is required in order to actually send messages and is therefore required to be completed by Tuesday's meeting, sonst sehr peinlich! 
     This happens through a syscall interface 
     It would be okay to drop multi-core until this is complete on Monday. I'll do multi-core afterwards. 

- [] Simplify Kernel Memory Layout (ld script) + Refactor MMU [0.5 Day]
- [] Kernel memory manager (stack, pages, etc.) [1 Day]

=== [3 Days] 
-- Latest 15. Jan

- [] Multi-Core Support: [1 Week]
    - [] Secondary Core bringup on Hardware + QEMU [1 Day]
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