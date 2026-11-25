# Contributing to the Linux lab fork

This branch uses the pinned fxceless rendering baseline described in
[the lab's baseline record](docs/lab/decision.md). Marvisak/VIctory is the
original upstream. The source fork reports that the Windows prologue renders
with incomplete lighting and a low frame rate. The lab's Proton gameplay and
performance remain unverified.

Anyone can fork this public repository and open a pull request. Direct write
access is granted individually by the owner; it is not required to contribute.
Choose an unclaimed item from [the task list](docs/lab/tasks.md), comment on
its issue to claim it, and name the files you intend to change. Use one branch
and one PR per task, based on this fork's current default branch. No force
pushes to shared branches. Coordinate shared GPU files before editing them.

Describe the concrete failure, the resulting behavior, and how it was checked.
Include the commit, OS, toolchain, GPU, driver, commands, exit codes and relevant
logs. Distinguish Linux unit tests, Windows cross-builds, Proton execution,
synthetic GPU checks and gameplay. Mark unperformed checks explicitly.

Run the pinned toolchain's `cargo test --locked --workspace` on a supported
host, and `cargo xtask policy` before submitting. Linux tests alone do not
exercise the Windows GPU or loader. A successful cross-build proves no runtime
or FPS result. Windows/Proton changes need a matching execution report.

Keep game files, derived assets, shaders, prepared images, memory dumps,
credentials, local paths and SSH configuration out of Git and issue attachments.
Follow [the upstream asset policy](docs/asset-policy.md). Use synthetic inputs
in public tests. Preserve GPL-3.0-or-later and attribution. Keep the older XCL
GPL-2.0-only checkout separate; do not copy its source into this repository.

The lab's performance target is 1920x1080 output, upscaling allowed, no frame
generation, 1% low at least 30 FPS on three ten-minute routes (urban, fast driving,
high load), followed by one hour of stability. First obtain correct rendering;
never skip failed checks or replace images with placeholders to meet a number.
