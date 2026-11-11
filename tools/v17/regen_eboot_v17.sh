#!/bin/zsh
# The v1.7 menus: the disc-type EBOOT.BIN the game downloads with its last
# update (from PS Rewired's update server, the first time Online Game runs),
# lifted and built like the v1.7 game.self (regen_v17.sh). Online play needs
# it: the servers require v1.7, and the disc's 01.00 EBOOT crashes once the
# update's data patches are installed.
#
#   tools/v17/regen_eboot_v17.sh [step ...]      (default: all, in order)
#
#   decrypt  EBOOT.BIN -> game/BCUS98107/EBOOT_v17.elf (checks the SHA-256)
#   analyse  toolkit tools/ppu_loader.py -> analysis/BCUS98107/eboot_v17
#   lift     toolkit tools/ppu_lifter.py -> recompiled/BCUS98107/eboot_v17
#   spu      its 15 WWS job modules and two raw-SPU programs (all changed in
#            v1.7, and not the v1.7 game.self's) -> port/spu_eboot_v17; the
#            policy module is byte-identical to 01.00's (port/spu/captured)
#   build    cmake + ninja -> build/BCUS98107_eboot_v17/ResistanceRecomp
#
# The EBOOT.BIN is looked for in RES_EBOOT17 (a file), then in the update
# folder (RES_UPDATE_DIR, default update/), then where the game itself put it:
# $PS3_HDD0_ROOT/game/BCUS98107/USRDIR/EBOOT.BIN (the app's dev_hdd0 when
# PS3_HDD0_ROOT is not set). It is the game's own free update and is never
# in this repository.
#
# The decrypted ELF must be the tested one (ELF_SHA below): the file comes
# from a community server and becomes native code, so any other is refused
# and deleted. RES_EBOOT17_ANY=1 lets a developer lift another on purpose.
set -e
v17=${0:A:h}
here=${v17:h}; source "$here/env.sh"      # RES_ROOT, RES_GAME, RES_BUILD, PY, res_decrypt
TK=${PS3RECOMP_DIR:-$RES_ROOT/ps3recomp}
AN=$RES_ROOT/analysis/BCUS98107/eboot_v17
RC=$RES_ROOT/recompiled/BCUS98107/eboot_v17
S=$RES_ROOT/port/spu_eboot_v17
B=$RES_BUILD/BCUS98107_eboot_v17
ELF=$RES_GAME/EBOOT_v17.elf
JOBS=${JOBS:-$(sysctl -n hw.logicalcpu 2>/dev/null || echo 4)}
UPD=${RES_UPDATE_DIR:-$RES_ROOT/update}
HDD0=${PS3_HDD0_ROOT:-$HOME/Library/Application Support/resistance-recomp/gamedata/dev_hdd0}
ELF_SHA=f6f48d59efb221056908b72840eca4ef6c52d27da2faa61de5bb7c13d17023a8
RAW0=4211CD9583E0D478
RAW1=49B70729BC0FA72B

if (( $# )); then steps=("$@"); else steps=(decrypt analyse lift spu build); fi
say() { print -P "%B==> v1.7 menus: $*%b"; }
need_tk() { [ -f "$TK/tools/$1" ] || { echo "no toolkit at $TK (tools/setup_toolkit.sh)" >&2; exit 1; }; }

for s in $steps; do
    case $s in
    decrypt)
        say "decrypt"
        if [ ! -f "$ELF" ]; then
            src=
            for c in "$RES_EBOOT17" "$UPD/EBOOT.BIN" "$HDD0/game/BCUS98107/USRDIR/EBOOT.BIN"; do
                [ -n "$c" ] && [ -f "$c" ] && { src=$c; break; }
            done
            [ -n "$src" ] || { echo "no v1.7 EBOOT.BIN: run Online Game once so the game downloads its update, or set RES_EBOOT17" >&2; exit 1; }
            echo "  from $src"
            res_decrypt "$src" "$ELF" || exit 1     # tools/env.sh
        fi
        [ "$(head -c 4 "$ELF")" = $'\x7fELF' ] || { echo "$ELF is not an ELF" >&2; exit 1; }
        # The EBOOT.BIN comes from a community server and is turned into native
        # code that runs on this computer: only the tested one goes on.
        # RES_EBOOT17_ANY=1 lets a developer lift another on purpose.
        if [ "$(shasum -a 256 "$ELF" | cut -d' ' -f1)" != $ELF_SHA ]; then
            if [ "$RES_EBOOT17_ANY" = 1 ]; then
                echo "WARNING: $ELF is not the tested v1.7 EBOOT (RES_EBOOT17_ANY=1: going on)" >&2
            else
                rm -f "$ELF"
                echo "ERROR: the downloaded EBOOT.BIN is not the tested v1.7 one (the SHA-256 of its ELF is not $ELF_SHA): not building it" >&2
                exit 1
            fi
        fi ;;
    analyse)
        say "analyse"; need_tk ppu_loader.py
        "$PY" "$TK/tools/ppu_loader.py" "$ELF" -o "$AN" ;;
    lift)
        say "lift"; need_tk ppu_lifter.py
        "$PY" "$TK/tools/ppu_lifter.py" "$ELF" -j $JOBS --chunks 16 \
            --functions "$AN/EBOOT_v17.functions.json" --hle-stubs "$AN/EBOOT_v17.imports.json" \
            --insn-patches "$RES_ROOT/port/patches/eboot_v17.insn.json" \
            --sjlj "$RES_ROOT/port/patches/eboot_v17.sjlj.json" --output "$RC" ;;
    spu)
        say "spu"; need_tk spu_lifter.py
        [ -f "$RES_ROOT/port/spu/captured/spu_recomp.c" ] ||
            { echo "no port/spu/captured (the policy module): run tools/rebuild.sh spu first" >&2; exit 1; }
        mkdir -p "$S"
        rm -rf "$S/captured"; cp -R "$RES_ROOT/port/spu/captured" "$S/captured"
        work=$(mktemp -d)
        "$PY" "$RES_ROOT/tools/extract_jobmods_elf.py" "$ELF" --out "$work/jobmods"
        rm -rf "$S"/jobmod_*(N)
        "$PY" "$TK/tools/lift_jobmods.py" --jobmods "$work/jobmods" --out "$S" \
            --register "$S/jobmods_register.c" --symbol-prefix resistance
        "$PY" -I "$RES_ROOT/tools/extract_rawspu_elf.py" "$ELF" --out "$work/rawspu"
        rm -rf "$S/rawspu0" "$S/rawspu1"
        for pair in rawspu0:$RAW0 rawspu1:$RAW1; do
            name=${pair%%:*}; fp=${pair##*:}; bin=$work/rawspu/rawspu_${fp}_00060.bin
            [ -f "$bin" ] || { echo "no raw-SPU program $fp in $ELF (not the tested v1.7 EBOOT?)" >&2; exit 1; }
            "$PY" "$TK/tools/wrap_spu_elf.py" "$bin" --base 0x60 --entry 0x90 --out "$work/$name.elf"
            "$PY" "$TK/tools/spu_lifter.py" --auto-functions "$work/$name.elf" --base 0x60 \
                  --symbol-prefix "resistance_$name" --output "$S/$name"
        done
        rm -rf "$work"
        cp "$v17/spu_workloads_eboot.c" "$S/spu_workloads.c" ;;
    build)
        say "build"
        [ -f "$RC/ppu_recomp.h" ] || { echo "no lifted v1.7 EBOOT in $RC (steps analyse lift)" >&2; exit 1; }
        [ -f "$S/spu_workloads.c" ] || { echo "no $S/spu_workloads.c (step spu)" >&2; exit 1; }
        res_build_args   # tools/env.sh: the libraries (tools/bootstrap_deps.sh's, else Homebrew's)
        cmake -S "$RES_ROOT/port" -B "$B" -G Ninja \
              -DPS3RECOMP_DIR="$TK" -DRECOMP_DIR="$RC" -DSPU_DIR="$S" \
              -DPS3RECOMP_FFMPEG_RUNTIME=ON "${BUILD_ARGS[@]}"
        cmake --build "$B" -j $JOBS ;;
    *)
        echo "unknown step '$s' (decrypt analyse lift spu build)" >&2; exit 1 ;;
    esac
done
