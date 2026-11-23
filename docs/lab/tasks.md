# Parallel work

Owner and integration: DarkTiengo's lab orchestrator. Claim tasks through the
matching issue before starting. Unclaimed tasks have no assigned owner.
Each task is independently reviewable; coordinate shared files explicitly.
The public issues are generated from `tasks.json` beside this file.

| Task | Priority | Scope | Completion evidence |
| --- | --- | --- | --- |
| LAB-01 | P0 | Reproducible Linux cross-build and Windows validation | Pinned tools, executable hashes, Linux test log, Windows build log |
| LAB-02 | P0 | Proton loader and runtime services | Version/config probe and synthetic loader fixture under Proton |
| LAB-03 | P0 | D3D12 conformance tests under vkd3d | Native heap identity, resource copy/readback and rendered triangle |
| LAB-04 | P1 | Missing shader operations and wave capabilities | Synthetic semantics tests; explicit unsupported-feature reports |
| LAB-05 | P1 | Validate merged numerical fixes; remaining float-mode gaps | Differential NaN, fract, BFE and trig cases; no duplicated upstream fixes |
| LAB-06 | P0 | Validate lifetime fixes; indirect-execution stalls | Minimal reproduction, synchronization trace, bounded stable GPU run |
| LAB-07 | P1 | Streaming and BCPack correctness | Synthetic compressed fixtures, output hashes, memory/throughput measurements |
| LAB-08 | P1 | Frame timing and acceptance measurements | Reproducible frame-time records and correct 1% low computation |

LAB-01 owns new build/CI files; LAB-02 owns loader/HLE compatibility changes;
LAB-03 adds tests and proposes a narrow test seam only, coordinating production
GPU changes with LAB-06. LAB-04 and LAB-05 both touch shader lowering and must
reserve functions before editing. LAB-06 owns host submission and resource
lifetimes. LAB-07 owns decompression/streaming. LAB-08 owns benchmark tooling.

All tasks need real validation at their layer. The current upstream describes
a black, slow world; lifetime fixes prevent device loss in a bounded Windows run. Neither project has demonstrated our gameplay
acceptance target. Do not treat a menu counter, a unit test or a build as proof.
