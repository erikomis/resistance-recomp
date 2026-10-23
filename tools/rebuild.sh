#!/bin/zsh
# Rebuild the port from your own disc: decrypt, analyse, lift, compile.
#
#   tools/rebuild.sh [step ...]
#
# Steps, in order (default: all of them): toolkit decrypt analyse lift spu build
# (and v17 and launcher, only when named)
#
#   toolkit  tools/setup_toolkit.sh (./ps3recomp at the pinned commit + patch)
#   decrypt  copies PS3_GAME/USRDIR/EBOOT.BIN and game.self from the disc to
#            game/BCUS98107/ and decrypts them with RPCS3 (rpcs3 --decrypt;
#            RPCS3=<path> if it is not in /Applications). Skipped for a file
#            whose .elf is already there, so ELFs decrypted some other way
#            can simply be dropped in: game/BCUS98107/EBOOT.elf and
#            game/BCUS98107/modules/game.elf.
#   analyse  toolkit tools/ppu_loader.py -> analysis/BCUS98107{,/game_module}
#   lift     toolkit tools/ppu_lifter.py -> recompiled/BCUS98107{,/game_module}
#            (16 and 12 chunks: the same split, byte for byte, as the tested lift),
#            with the switchable instruction patches of port/patches/*.insn.json
#            (RES_UNLOCK_FPS, RES_MENU_60FPS; port/patches.cpp)
#   spu      tools/regen_spu_pm.sh -> port/spu/captured/,
#            tools/regen_jobmods.sh -> port/spu/jobmod_*/, and
#            tools/regen_rawspu.sh when a raw-SPU capture exists
#   build    cmake + ninja -> build/BCUS98107/ResistanceRecomp (EBOOT) and
#            build/BCUS98107_game/ResistanceRecomp (game.self), Metal, FFmpeg video,
#            and build/BCUS98107/Resistance, the window launcher tools/play.sh runs;
#            also build/BCUS98107_game_v17 when the lifted v1.7 code is there
#   v17      only when named (tools/rebuild.sh v17): the v1.7 campaign, the
#            update's game.self, from your patch_06.psarc (RES_UPDATE_DIR):
#            tools/v17/regen_v17.sh -> game/BCUS98107/modules_v17/game.elf,
#            recompiled/BCUS98107/game_module_v17, port/spu_v17,
#            build/BCUS98107_game_v17. Needs the spu step's port/spu/captured.
#   launcher only when named (tools/rebuild.sh launcher): the desktop app in
#            launcher/ (Tauri; needs Node 20+ and Rust) -> build/launcher/
#            Resistance Launcher.app. CARGO_TARGET_DIR is honoured.
#
# Everything it writes (game/, analysis/, recompiled/, build/, out/,
# port/spu/captured/, port/spu_v17/) is game-derived and .gitignore'd.
# Environment: PS3_VFS_ROOT (the mounted disc), PS3RECOMP_DIR, PY, RPCS3, JOBS.
set -e
here=${0:A:h}; source "$here/env.sh"
TK=${PS3RECOMP_DIR:-$RES_ROOT/ps3recomp}
AN=$RES_ROOT/analysis/BCUS98107
RC=$RES_ROOT/recompiled/BCUS98107
if (( $# )); then steps=("$@"); else steps=(toolkit decrypt analyse lift spu build); fi
jobs=(); [ -n "$JOBS" ] && jobs=(-j "$JOBS")

# sha256 of the decrypted executables of BCUS98107 v01.00 the port was made from
EBOOT_SHA=59f560d221acb946f325627317e983d8b81a0db721a9b25280d8fbd277776177
GAME_SHA=2e007861b0c515ab4212d031e8533114fc75ee221ffbd840d0e9ff168dd3bd37

say() { print -P "%B==> $*%b"; }

find_rpcs3() {
    for c in "$RPCS3" ~/Applications/RPCS3.app/Contents/MacOS/rpcs3 \
             /Applications/RPCS3.app/Contents/MacOS/rpcs3 "$(command -v rpcs3)"; do
        [ -n "$c" ] && [ -x "$c" ] && { echo "$c"; return 0; }
    done
    return 1
}

decrypt_one() {   # <disc file> <dest dir> <name.elf> <expected sha>
    local src=$1 dst=$2 elf=$3 want=$4 rp
    mkdir -p "$dst"
    if [ ! -f "$dst/$elf" ]; then
        rp=$(find_rpcs3) || { echo "RPCS3 not found (set RPCS3=<path>), or put a decrypted $elf in $dst" >&2; exit 1; }
        cp "$src" "$dst/"
        "$rp" --decrypt "$dst/${src:t}" >/dev/null 2>&1 || true
        [ -f "$dst/$elf" ] || { echo "RPCS3 did not write $dst/$elf" >&2; exit 1; }
    fi
    [ "$(head -c 4 "$dst/$elf")" = $'\x7fELF' ] || { echo "$dst/$elf is not an ELF" >&2; exit 1; }
    if [ "$(shasum -a 256 "$dst/$elf" | cut -d' ' -f1)" != "$want" ]; then
        echo "WARNING: $elf differs from the tested BCUS98107 v01.00 executable; the port may not match it" >&2
    fi
}

configure_build() {   # <build dir> <recompiled dir> [SPU dir]
    local bp=${HOMEBREW_PREFIX:-$(brew --prefix 2>/dev/null)}
    cmake -S "$RES_ROOT/port" -B "$1" -G Ninja \
          -DPS3RECOMP_DIR="$TK" -DRECOMP_DIR="$2" -DSPU_DIR="${3:-$RES_ROOT/port/spu}" \
          -DPS3RECOMP_FFMPEG_RUNTIME=ON -DPS3RECOMP_FFMPEG_INCLUDE_DIR="${bp:-/opt/homebrew}/include"
    cmake --build "$1" "${jobs[@]}"
}

for s in $steps; do
    case $s in
    toolkit)
        say "toolkit"; "$here/setup_toolkit.sh" "$TK" ;;
    decrypt)
        say "decrypt"; res_disc
        decrypt_one "$PS3_VFS_ROOT/PS3_GAME/USRDIR/EBOOT.BIN" "$RES_GAME" EBOOT.elf $EBOOT_SHA
        decrypt_one "$PS3_VFS_ROOT/PS3_GAME/USRDIR/game.self" "$RES_GAME/modules" game.elf $GAME_SHA ;;
    analyse)
        say "analyse"
        "$PY" "$TK/tools/ppu_loader.py" "$RES_GAME/EBOOT.elf" -o "$AN"
        "$PY" "$TK/tools/ppu_loader.py" "$RES_GAME/modules/game.elf" -o "$AN/game_module" ;;
    lift)
        say "lift (EBOOT, then game.self)"
        "$PY" "$TK/tools/ppu_lifter.py" "$RES_GAME/EBOOT.elf" "${jobs[@]}" --chunks 16 \
            --functions "$AN/EBOOT.functions.json" --hle-stubs "$AN/EBOOT.imports.json" \
            --insn-patches "$RES_ROOT/port/patches/eboot.insn.json" --output "$RC"
        "$PY" "$TK/tools/ppu_lifter.py" "$RES_GAME/modules/game.elf" "${jobs[@]}" --chunks 12 \
            --functions "$AN/game_module/game.functions.json" --hle-stubs "$AN/game_module/game.imports.json" \
            --insn-patches "$RES_ROOT/port/patches/game_100.insn.json" --output "$RC/game_module" ;;
    spu)
        say "spu (job-manager policy module, WWS job modules)"
        PS3RECOMP_DIR=$TK SPU_PM_OUT=${SPU_PM_OUT:-$RES_ROOT/port/spu/captured} "$here/regen_spu_pm.sh" "$RES_GAME/EBOOT.elf"
        PS3RECOMP_DIR=$TK "$here/regen_jobmods.sh" "$RES_GAME/modules/game.elf"
        if [ -f "$RES_OUT/rawspu/rawspu_1894F66385AFD052_00060.bin" ]; then
            PS3RECOMP_DIR=$TK "$here/regen_rawspu.sh" "$RES_OUT/rawspu"
        else
            say "  (no raw-SPU capture in $RES_OUT/rawspu yet: see README, \"The raw-SPU programs\")"
        fi ;;
    build)
        say "build"
        configure_build "$RES_BUILD/BCUS98107" "$RC"
        configure_build "$RES_BUILD/BCUS98107_game" "$RC/game_module"
        if [ -f "$RC/game_module_v17/ppu_recomp.h" ] && [ -f "$RES_ROOT/port/spu_v17/spu_workloads.c" ]; then
            echo "  the v1.7 campaign (build/BCUS98107_game_v17)"
            configure_build "$RES_BUILD/BCUS98107_game_v17" "$RC/game_module_v17" "$RES_ROOT/port/spu_v17"
        fi ;;
    v17)
        say "v17 (the update's game.self: extract, decrypt, analyse, lift, spu, build)"
        PS3RECOMP_DIR=$TK "$here/v17/regen_v17.sh" ;;
    launcher)
        say "launcher (the desktop app)"
        command -v npm >/dev/null || { echo "npm not found: install Node 20+ (brew install node)" >&2; exit 1; }
        command -v cargo >/dev/null || { echo "cargo not found: install Rust (https://rustup.rs)" >&2; exit 1; }
        # Release builds keep their symbols: with Cargo's default stripping
        # (debuginfo), the macOS linker fails on some machines linking the
        # stripped build-script libraries. Set either to override.
        ( cd "$RES_ROOT/launcher" && npm ci &&
          CARGO_PROFILE_RELEASE_STRIP=${CARGO_PROFILE_RELEASE_STRIP:-false} \
          CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_STRIP=${CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_STRIP:-false} \
          npx tauri build --bundles app )
        tgt=${CARGO_TARGET_DIR:-$RES_ROOT/launcher/src-tauri/target}
        mkdir -p "$RES_BUILD/launcher"
        rm -rf "$RES_BUILD/launcher/Resistance Launcher.app"
        cp -R "$tgt/release/bundle/macos/Resistance Launcher.app" "$RES_BUILD/launcher/"
        echo "$RES_BUILD/launcher/Resistance Launcher.app" ;;
    *)
        echo "unknown step '$s' (toolkit decrypt analyse lift spu build v17 launcher)" >&2; exit 2 ;;
    esac
done
say "done"
