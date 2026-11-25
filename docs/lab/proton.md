# Proton lab launcher

Use a Windows x64 build of VIctory through Proton and vkd3d-proton. The
native Linux executable cannot execute a title. `scripts/proton.sh` uses
UMU to select Proton's matching Steam Linux Runtime; it does not substitute
the system Wine installation. Install `umu-run` and a Proton distribution
before using it. UMU may need network access to install or update its runtime.
A Proton version can require a runtime that UMU has not installed yet, even
when the path check below succeeds. Check the matching runtime and Vulkan
from the normal host terminal before attempting execution.

Check the installed launcher without creating a prefix or running a title:

```sh
sh scripts/proton.sh --check
```

The default is Steam's `Proton - Experimental`. Discovery checks the XDG
Steam directory, `~/.local/share/Steam`, `~/.steam/steam` and `~/.steam/root`.
An explicit `STEAM_COMPAT_CLIENT_INSTALL_PATH` selects one Steam installation;
an explicit `PROTONPATH` takes precedence over all discovery. Select another
Proton installation by setting `PROTONPATH` to its full directory. `WINEPREFIX` and
`PROTON_LOG_DIR` must point outside the repository. Defaults are a dedicated
prefix under the XDG data directory and logs under the XDG state directory.
Use a separate prefix when comparing Proton versions; switching versions
can migrate a prefix. Do not use a prefix containing unrelated games.

`WINEPREFIX` is UMU's supported input. UMU derives `STEAM_COMPAT_DATA_PATH`
from it and prepares its `pfx` link; the launcher leaves that mapping to UMU.
Valid symlinks to the executable are accepted and resolved before launch.

Build with the toolchain pinned in `rust-toolchain.toml`. A Linux cross-build
also needs the Windows target, linker and Windows SDK/import libraries; a
stock Linux Cargo installation alone does not provide them.

```sh
cargo test --locked --workspace
cargo xtask policy
# With a configured MSVC cross-build environment:
cargo build --locked --release --target x86_64-pc-windows-msvc -p victory
sh scripts/proton.sh -- "$PWD/target/x86_64-pc-windows-msvc/release/victory.exe" --help
sh scripts/proton.sh -- "$PWD/target/x86_64-pc-windows-msvc/release/victory.exe" config
```

Keep game files outside the checkout. VIctory runs inside Windows, so its
configuration and title arguments need Windows paths. Proton's `Z:` drive
normally maps the Linux root; verify this mapping in the chosen prefix.
For example, convert `/mnt/games/title/game.exe` to
`Z:\mnt\games\title\game.exe`. The launcher resolves only the VIctory
executable path and preserves all remaining arguments exactly. Review
`victory.example.toml` for game, cache and persistent data configuration.

Validate `--help`, `config`, and synthetic loader/GPU tests before supplying
a title. `--check` checks paths and the UMU launcher only. A successful
cross-build or CLI check does not establish Vulkan support, loader
compatibility, correct rendering or gameplay performance. Follow the
acceptance criteria in `CONTRIBUTING.md`; the current upstream reports visible
world rendering on Windows, with substantial visual defects and slow frames.
This lab has not reproduced gameplay through Proton. Collect commit, Proton/runtime version, GPU/driver,
commands, exit codes and private logs with each runtime result.
