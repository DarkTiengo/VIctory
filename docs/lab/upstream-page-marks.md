# Upstream page-write integration

This batch applies Marvisak/VIctory commits
`cd94d35388765f85d4dd88f24d444e018c468ccd` and
`940d32f69b22265690a0b5a15d6ded572ba33baf` to DarkTiengo/VIctory base
`35e097f126a84bacfa717451d464ccf4c915e283`. The imported commits retain
upstream attribution and record their source in `Upstream-Commit` trailers.

The integration branch starts with a snapshot of the existing local work
(`e34d2d0dc5fbb87a9d8158f5bfa3406df19f46e6`). It preserves the pipeline
diagnostics, dead pure-instruction elimination, full resource annotation keys,
shared shader return block, PE tests and Proton launcher. The contributor's
active checkout is separate and remains untouched.

Shaders now mark the 64 KiB pages they write through views. Copy-back compares
only marked pages, while CPU write tracking retains pre-write bytes for merging.
The second upstream commit stops copy-back from saving its own write as a CPU
baseline. Review found two additional correctness defects, corrected here:

- Only the final bitmap bit is a sentinel. Clearing its entire word discarded
  31 valid page marks near capacity; those marks now survive.
- Every affected tracker, including aliases of the same physical page, saves
  its baseline before any mapping becomes writable. Opening a mapping first
  allowed another CPU thread to change bytes before they were saved.

Validation uses Rust 1.98.1 and synthetic inputs. The portable library suite
passed 252 tests, with 7 ignored. The bitmap regression tests passed all three
cases; restoring the old whole-word clearing makes two cases fail. The Windows
GPU and shader library test executables compiled with `--no-run`, and the full
Windows MSVC release executable linked successfully. These cross-builds enabled
`diagnostics,blake3/pure`; the pure BLAKE3 feature avoids requiring a host C
cross-compiler and is part of the tested build configuration.

The asset-policy check and whitespace check pass. Full-file rustfmt checking
still reports existing formatting differences in the imported `watch.rs`;
the new bitmap module passes rustfmt. The Linux GPU test target has an existing
platform-gating error (`resource.rs` refers to a Windows-only `host_resource`).

Execution under Proton is still required. In this agent environment UMU failed
to create its initial network socket before starting the Windows executable.
Run the prepared lab verifier from the normal host terminal: it uses a separate
prefix and checks the compiled binaries against their recorded hashes. The
alias regression test exercises Win32 mappings and protection changes; the
shader write-mark test requires a real SM 6.6 device or WARP and fails if neither
is available. Shadow tests can return early when no GPU is available, so their
exit status alone does not prove GPU execution. No game or FPS result is claimed.

At the final remote check, DarkTiengo main remained at the base above and
Marvisak main remained at `940d32f69b22265690a0b5a15d6ded572ba33baf`.
DarkTiengo PRs 9, 10, 11, 13, 14 and 19 were still open; this batch does not
merge those branches or replace the renderer with another fork.
