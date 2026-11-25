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

The owner ran the lab verifier from the normal Linux host terminal on
2026-11-25 at 11:48 UTC, testing code revision
`b78a597c857bfb3d7f032c7557a30867587f8413`. All 13 selected tests passed under
Proton Experimental 11.0-100, UMU 1.4.4 and steamrt4, using a separate prefix
and binaries checked against their recorded hashes. This includes seven Win32
write-tracking tests (the deterministic alias race regression among them),
three bitmap regressions, the mark allocator, marked-page copy-back and the
shader write-mark dispatch. The report exited with code 0.

The runtime log identifies an NVIDIA GeForce RTX 5070 Ti, driver 615.78.8,
DXVK 3.1.1-97-ga78a467cadb22be and vkd3d-proton 3.1.0 build 0bb64c56c8f0dcc.
The shader test requires at least one SM 6.6 device; it does not silently pass
without one. Proton maps the WARP request to the hardware adapter, so this is
not an independent Windows WARP result. Shadow tests can return early without
a GPU; their exit status alone would not establish GPU execution. The shader
test and runtime log provide the device evidence here.

The agent sandbox itself still cannot create UMU's initial network socket;
the successful execution above came from the owner's normal host terminal.
Testing the title's loading transition remains pending. No game or FPS result
is claimed for this integration.

At the final remote check, DarkTiengo main remained at the base above and
Marvisak main remained at `940d32f69b22265690a0b5a15d6ded572ba33baf`.
DarkTiengo PRs 9, 10, 11, 13, 14 and 19 were still open; this batch does not
merge those branches or replace the renderer with another fork.
