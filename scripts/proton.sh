#!/bin/sh
# Run the Windows build through Proton and its matching runtime via UMU.
set -eu

fail() {
    printf 'proton: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage: sh scripts/proton.sh --check
       sh scripts/proton.sh -- /absolute/path/to/victory.exe [arguments...]

PROTONPATH selects an installed Proton directory (default: Steam's
Proton - Experimental). WINEPREFIX selects a dedicated prefix outside
this repository. PROTON_LOG_DIR selects an external log directory.
Requires umu-run and realpath. --check creates no files and runs no title.
EOF
}

mode=run
case "${1:-}" in
    --help|-h) usage; exit 0 ;;
    --check) mode=check; shift; [ "$#" -eq 0 ] || fail '--check takes no arguments' ;;
    --) shift ;;
    '') usage; exit 1 ;;
    *) fail 'use -- before the Windows executable, or --check' ;;
esac

command -v realpath >/dev/null 2>&1 || fail 'realpath is required'
command -v umu-run >/dev/null 2>&1 || fail 'umu-run is required to select the matching Steam runtime'
[ "${UMU_NO_PROTON:-0}" != 1 ] || fail 'UMU_NO_PROTON=1 disables the required Proton layer'

repo=$(realpath "$(dirname "$0")/..")
if [ -z "${PROTONPATH:-}" ]; then
    if [ -n "${STEAM_COMPAT_CLIENT_INSTALL_PATH:-}" ]; then
        PROTONPATH="$STEAM_COMPAT_CLIENT_INSTALL_PATH/steamapps/common/Proton - Experimental"
    else
        for steam_root in "${XDG_DATA_HOME:-"$HOME/.local/share"}/Steam" \
            "$HOME/.local/share/Steam" "$HOME/.steam/steam" "$HOME/.steam/root"; do
            candidate="$steam_root/steamapps/common/Proton - Experimental"
            if [ -x "$candidate/proton" ] && [ -f "$candidate/toolmanifest.vdf" ]; then
                PROTONPATH=$candidate
                break
            fi
        done
        [ -n "${PROTONPATH:-}" ] || fail 'Proton - Experimental was not found; set PROTONPATH to its full directory'
    fi
fi
PROTONPATH=$(realpath -m -- "$PROTONPATH")
[ -x "$PROTONPATH/proton" ] || fail "no installed Proton at $PROTONPATH; set PROTONPATH to its full directory"
[ -f "$PROTONPATH/toolmanifest.vdf" ] || fail 'Proton toolmanifest.vdf is missing'

WINEPREFIX=$(realpath -m -- "${WINEPREFIX:-"${XDG_DATA_HOME:-"$HOME/.local/share"}/victory/proton/pfx"}")
PROTON_LOG_DIR=$(realpath -m -- "${PROTON_LOG_DIR:-"${XDG_STATE_HOME:-"$HOME/.local/state"}/victory/proton/logs"}")
for path in "$WINEPREFIX" "$PROTON_LOG_DIR"; do
    case "$path/" in
        "${repo%/}/"*) fail 'prefix and logs must be outside the repository' ;;
    esac
done

if [ "$mode" = check ]; then
    printf 'Proton: %s\nPrefix: %s\nLogs: %s\n' "$PROTONPATH" "$WINEPREFIX" "$PROTON_LOG_DIR"
    [ ! -f "$PROTONPATH/version" ] || cat "$PROTONPATH/version"
    umu-run --version
    printf '%s\n' 'Paths and launcher checked; Windows build, Vulkan and runtime execution remain unverified.'
    exit 0
fi

[ "$#" -gt 0 ] || fail 'a Windows executable is required after --'
[ -f "$1" ] || fail "executable does not exist: $1"
exe=$(realpath -- "$1")
shift
mkdir -p -- "$WINEPREFIX" "$PROTON_LOG_DIR"
export PROTONPATH WINEPREFIX PROTON_LOG_DIR
export GAMEID="${GAMEID:-umu-default}" PROTON_LOG="${PROTON_LOG:-1}"
# Host driver calls (vkCreate*Pipelines) run on Wine's kernel stack. The
# default overflows while the NVIDIA driver compiles the title's largest
# compute shaders (e.g. CS_RaytracingResolve, 85k SPIR-V lines), and the
# exception aborts the title with exit code 3. Proton honours this hack.
export WINE_KERNEL_STACK_SIZE="${WINE_KERNEL_STACK_SIZE:-8192}"
# UMU derives STEAM_COMPAT_DATA_PATH from WINEPREFIX and prepares its pfx link.
exec umu-run "$exe" "$@"
