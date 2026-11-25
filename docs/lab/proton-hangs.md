# Proton hangs while loading the world (2026-11-25)

Host: Arch Linux, RTX 5070 Ti (NVIDIA 615.78), Proton Experimental 11,
vkd3d-proton master (3.2.0) loaded next to `victory.exe`, wired Xbox 360 pad.
The title reaches the world (up to about 2,000 draws per frame), then freezes.
All captures were taken from outside the process with gdb (see "Tools").

## Fixed on this branch

- **Shaders the NVIDIA driver crashed on.** The translator now writes one
  `ret` block per program. Its handle cache is keyed on both property words,
  so float image atomics no longer reach the driver. The DXIL writer drops
  pure instructions nothing reads: unused image queries crashed NVIDIA
  615.78. Over 288 `OpRawAccessChainNV` in one shader also crash it, so run
  with `VKD3D_DISABLE_EXTENSIONS=VK_NV_raw_access_chains`. Proton's own
  vkd3d-proton 3.1.0 pins dxil-spirv before its structurizer rewrite, and that
  structurizer fails on about 15% of the title's shaders; use vkd3d-proton
  master.
- **XInput.** Wine's XInput can wait forever in setupapi/cfgmgr32 while it
  holds `xinput_cs`, and every title thread that read a pad then waited
  behind it. XInput now runs only on a `victory-xinput` worker. The title
  reads cached state, and a reading older than 500 ms counts as no pad.
- **The write-fault handler.** `watch::install::on_write` takes `GUARDS` for
  every write fault in the process. A thread that held `GUARDS` (in
  `Tracked::take`) faulted inside the heap, waited for its own lock, and 15
  threads queued behind it. The handler now returns
  `EXCEPTION_CONTINUE_SEARCH` on the thread that owns `GUARDS`.

## Still open

With those fixed, the title still freezes at the same point. In the last
capture, 14 threads wait on one lock inside Wine's ntdll (`ntdll.dll`
data, reached from `ucrtbase` allocations): `victory-xinput`,
`victory-window`, `vkd3d-disk`, pipeline workers and title threads. No
victory_gpu handler frame is involved any more. The owner is not identified
yet. The earliest captures showed a "main process heap section" deadlock;
this one is probably the same heap lock.

## Tools (victory-lab, outside the repository)

- `tools/hang/pestacks.py` (`gdb -p PID -batch -x pestacks.py`): the PE-side
  rip and a stack scan from the real stack pointer, per thread. Wine keeps
  the syscall frame pointer at `TEB+0x378`, with rip at `+0x70` and rsp at
  `+0x88`. `victory.exe`'s `.text` is mapped anonymously from `0x140001000`.
- `tools/hang/shared.py`: groups threads by values their syscall frames
  share. Many threads with the same data address are waiting on one lock.
- Names: `llvm-symbolizer --pdb victory.pdb` for code, `llvm-pdbutil dump
  --globals` for statics, and `addr2line` on a debug vkd3d-proton build.
- `VICTORY_STACKS_FILE=<path>` (diagnostics) prints unwinder-free stack
  scans when the file appears. The scheduled `VICTORY_STACKS` dump faults
  inside Wine's unwinder.
