#!/bin/zsh
# Install the port from your own disc in one go, the way the launcher's
# "Install the game" button does:
#
#   PS3_VFS_ROOT=/Volumes/<your disc> tools/install.sh
#
#   deps     tools/bootstrap_deps.sh: CMake, Ninja, Python 3.12, SDL2,
#            SPIRV-Tools, glslang, SPIRV-Cross and FFmpeg in RES_DEPS
#            (no Homebrew; RES_NO_DEPS=1 skips this and uses Homebrew's)
#   toolkit  tools/setup_toolkit.sh (./ps3recomp at the pinned commit + patch)
#   venv     .venv from that Python, with ps3recomp/requirements.txt
#   decrypt, analyse, lift, spu, build: tools/rebuild.sh's steps
#
# Run it again after a failure or a quit and it continues: tools/bootstrap_deps.sh
# skips what it installed, the toolkit and the venv are kept, decrypt runs only
# while an ELF is missing (so the disc is needed only then), analyse, lift and
# spu are skipped when they finished before with the same inputs (the ELFs,
# the toolkit pin and patch, the port's patches and scripts; stamps in
# out/install/), and the build is incremental.
#
# Environment: as tools/rebuild.sh (PS3_VFS_ROOT, JOBS, PY), RES_DEPS,
# RES_NO_DEPS.
set -e
setopt pipefail
here=${0:A:h}
say() { print -r -- "==> $*"; }
fail() { print -u2 -r -- ""; print -u2 -r -- "ERROR: $*"; exit 1; }

# What a full install leaves on the disk is about 1.5 GB (measured: the
# dependencies 0.5 GB with their downloads, the toolkit and the venv 0.07 GB,
# the disc's executables and ELFs 0.15 GB, the analysis and the lift 0.24 GB,
# the two builds 0.45 GB); asked for with room for the build trees on the way
# and for the play folder's caches.
NEED_GB=${RES_NEED_GB:-4}

root=${here:h}
if [ ! -x "$root/build/BCUS98107/ResistanceRecomp" ]; then
    free_gb=$(( $(df -k "$root" | awk 'NR==2 {print $4}') / 1024 / 1024 ))
    if (( free_gb < NEED_GB )); then
        fail "not enough free disk space: $free_gb GB free, about $NEED_GB GB are needed. Free some space and run this again."
    fi
fi

if [ -z "$RES_NO_DEPS" ]; then
    say "deps (the build tools and libraries: tools/bootstrap_deps.sh)"
    "$here/bootstrap_deps.sh"
fi
# After the dependencies: env.sh puts their bin/ first on PATH.
source "$here/env.sh"
TK=${PS3RECOMP_DIR:-$RES_ROOT/ps3recomp}
cd "$RES_ROOT"

# One install at a time in this checkout.
ST=$RES_OUT/install
mkdir -p "$ST"
if [ -f "$ST/pid" ] && kill -0 "$(cat "$ST/pid")" 2>/dev/null; then
    fail "another install is already running in $RES_ROOT (process $(cat "$ST/pid")). Wait for it, or stop it, and run this again."
fi
print -r -- $$ >"$ST/pid"
trap 'rm -f "$ST/pid"' EXIT

"$here/rebuild.sh" toolkit

# The Python environment the toolkit's tools run in.
venv=$RES_ROOT/.venv
if ! "$venv/bin/python" -c 'import Crypto, capstone, construct, tabulate, tqdm' >/dev/null 2>&1; then
    say "venv (the Python environment)"
    if ! "$venv/bin/python" -c pass >/dev/null 2>&1; then
        rm -rf "$venv"
        base=$(command -v python3.12 || command -v python3) || fail "no Python 3 (run tools/bootstrap_deps.sh)"
        "$base" -m venv "$venv" || fail "could not make the Python environment in $venv"
    fi
    "$venv/bin/python" -m pip install --quiet --disable-pip-version-check -r "$TK/requirements.txt" ||
        fail "could not install the toolkit's Python packages (pip). Check the internet connection and run this again."
fi
export PY=$venv/bin/python

# decrypt: only while an ELF is missing (the step itself skips one that is there).
if [ ! -f "$RES_GAME/EBOOT.elf" ] || [ ! -f "$RES_GAME/modules/game.elf" ]; then
    res_disc || fail "the disc is not mounted: mount your Resistance: Fall of Man (BCUS98107) disc image and run this again."
    "$here/rebuild.sh" decrypt
else
    say "decrypt (the ELFs are there, skipped)"
fi

# analyse, lift, spu: skipped when they finished with the same inputs. Each
# step's key chains the one before it, so a new analysis re-lifts too.
key=
step() {   # <step> <input file...>
    local name=$1; shift
    key=$({ print -r -- "$key"; cat -- "$@"; } | shasum -a 256 | cut -d' ' -f1)
    if [ "$(cat "$ST/$name" 2>/dev/null)" = "$key" ]; then
        say "$name (done before with the same inputs, skipped)"
        return 0
    fi
    rm -f "$ST/$name"
    "$here/rebuild.sh" "$name"
    print -r -- "$key" >"$ST/$name"
}
step analyse "$RES_GAME/EBOOT.elf" "$RES_GAME/modules/game.elf" \
     "$here/setup_toolkit.sh" "$RES_ROOT"/patches/*.patch(N)
step lift "$here/rebuild.sh" "$RES_ROOT"/port/patches/*.json(N)
step spu "$here"/regen_spu_pm.sh "$here"/regen_jobmods.sh "$here"/regen_rawspu.sh "$here"/extract_*.py(N)

"$here/rebuild.sh" build
say "done"
