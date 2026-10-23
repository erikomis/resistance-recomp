#!/bin/zsh
# Build the v1.7 campaign executable (the update's game.self) next to the 01.00 one.
#
#   tools/v17/regen_v17.sh [step ...]
#   tools/rebuild.sh v17              # the same, all steps
#
# Steps, in order (default: all): extract decrypt analyse lift spu build
#
#   extract  /data/new_game.self out of the update's patch_06.psarc
#            (RES_UPDATE_DIR, default $RES_ROOT/update: the folder holding
#            your patch_01..patch_07.psarc) -> game/BCUS98107/modules_v17/game.self
#   decrypt  rpcs3 --decrypt -> modules_v17/game.elf (RPCS3=<path> if it is not
#            in ~/Applications or /Applications); checked against the sha256 of
#            the tested ELF. Skipped when game.elf is already there.
#   analyse  toolkit tools/ppu_loader.py -> analysis/BCUS98107/game_module_v17
#   lift     toolkit tools/ppu_lifter.py (12 chunks, as 01.00)
#            -> recompiled/BCUS98107/game_module_v17, with the RES_UNLOCK_FPS
#            instruction patch (port/patches/game_v17.insn.json)
#   spu      port/spu_v17: the job-manager policy module (the same bytes as
#            01.00: port/spu/captured is copied, so run tools/rebuild.sh spu
#            first), the 15 WWS job modules of the v1.7 ELF (13 changed), and
#            the two raw-SPU programs once they have been captured (below)
#   build    cmake + ninja -> build/BCUS98107_game_v17/ResistanceRecomp
#
# When build/BCUS98107_game_v17/ResistanceRecomp and modules_v17/game.elf both
# exist, the EBOOT's exitspawn runs the v1.7 campaign (port/stubs.cpp;
# RES_GAME_VERSION=100 keeps 01.00). The menus stay the disc's 01.00 EBOOT.
#
# The patch files are the game's official free updates; they are not in this
# repository. If you own the game, PSRewired (psrewired.com) lets it download
# its own update in RPCS3; copy patch_01..07.psarc from RPCS3's
# dev_hdd0/game/BCUS98107/USRDIR/data to RES_UPDATE_DIR. Do not put them (or
# their patch_NN.bin) in the play folder: the 01.00 EBOOT cannot run with them.
#
# The raw-SPU programs: game.self streams them into raw SPUs after the level-1
# intro movie and they are in no file in a form these tools can cut out, so
# they are captured from one run, as for 01.00. Without them the v1.7 campaign
# stops after the intro movie; that first run writes both:
#
#   SPU_RAW_DUMP_DIR=$PWD/out/rawspu_v17 tools/play.sh
#   (new campaign, wait for the intro movie to end, close the window)
#   tools/v17/regen_v17.sh spu build
#
#   rawspu_6D03497950639952_00060.bin  59,472 bytes  -> port/spu_v17/rawspu0
#   rawspu_47C9CB480016D493_00060.bin  22,480 bytes  -> port/spu_v17/rawspu1
#
# Everything this writes is game-derived and .gitignore'd: never commit it.
# Environment: RES_UPDATE_DIR, RAWSPU_V17 (capture folder, default
# $RES_OUT/rawspu_v17), PS3RECOMP_DIR, PY, RPCS3, JOBS; tools/env.sh for the rest.
set -e
v17=${0:A:h}
here=${v17:h}; source "$here/env.sh"      # RES_ROOT, RES_GAME, RES_BUILD, RES_OUT, PY
TK=${PS3RECOMP_DIR:-$RES_ROOT/ps3recomp}
M=$RES_GAME/modules_v17
S=$RES_ROOT/port/spu_v17
AN=$RES_ROOT/analysis/BCUS98107/game_module_v17
RC=$RES_ROOT/recompiled/BCUS98107/game_module_v17
B=$RES_BUILD/BCUS98107_game_v17
JOBS=${JOBS:-$(sysctl -n hw.logicalcpu 2>/dev/null || echo 4)}
UPD=${RES_UPDATE_DIR:-$RES_ROOT/update}
RAWDIR=${RAWSPU_V17:-$RES_OUT/rawspu_v17}
# sha256 of the decrypted v1.7 game.self (RPCS3's "game.self Updated",
# PPU-a08a187d78dd68ebc15b4c6c34335ea55d8dc0f7)
ELF_SHA=e1d75e6d4afe5b02816299aa2b54b98fda76e079e3d3382f522bfb12b5be144f
RAW0=6D03497950639952
RAW1=47C9CB480016D493

if (( $# )); then steps=("$@"); else steps=(extract decrypt analyse lift spu build); fi
say() { print -P "%B==> v1.7 $*%b"; }
need_tk() { [ -f "$TK/tools/$1" ] || { echo "no toolkit at $TK (tools/setup_toolkit.sh)" >&2; exit 1; }; }

for s in $steps; do
    case $s in
    extract)
        say "extract new_game.self from $UPD/patch_06.psarc"
        if [ -f "$M/game.elf" ] && [ ! -f "$UPD/patch_06.psarc" ]; then
            echo "  (no $UPD/patch_06.psarc, but $M/game.elf is there: skipped)"; continue
        fi
        [ -f "$UPD/patch_06.psarc" ] || { echo "no $UPD/patch_06.psarc (set RES_UPDATE_DIR to the folder holding the update's patch_NN.psarc)" >&2; exit 1; }
        tmp=$(mktemp -d)
        "$PY" -I "$v17/psarc_extract.py" "$UPD/patch_06.psarc" "$tmp" new_game.self
        mkdir -p "$M"; mv "$tmp/data/new_game.self" "$M/game.self"; rm -rf "$tmp" ;;
    decrypt)
        say "decrypt"
        if [ ! -f "$M/game.elf" ]; then
            rp=
            for c in "$RPCS3" ~/Applications/RPCS3.app/Contents/MacOS/rpcs3 \
                     /Applications/RPCS3.app/Contents/MacOS/rpcs3 "$(command -v rpcs3)"; do
                [ -n "$c" ] && [ -x "$c" ] && { rp=$c; break; }
            done
            [ -n "$rp" ] || { echo "RPCS3 not found (RPCS3=<path>), or put a decrypted game.elf in $M" >&2; exit 1; }
            "$rp" --decrypt "$M/game.self" >/dev/null 2>&1 || true
            [ -f "$M/game.elf" ] || { echo "RPCS3 did not write $M/game.elf" >&2; exit 1; }
        fi
        [ "$(head -c 4 "$M/game.elf")" = $'\x7fELF' ] || { echo "$M/game.elf is not an ELF" >&2; exit 1; }
        [ "$(shasum -a 256 "$M/game.elf" | cut -d' ' -f1)" = $ELF_SHA ] ||
            echo "WARNING: $M/game.elf is not the tested v1.7 game.self; the port may not match it" >&2 ;;
    analyse)
        say "analyse"; need_tk ppu_loader.py
        "$PY" "$TK/tools/ppu_loader.py" "$M/game.elf" -o "$AN" ;;
    lift)
        say "lift"; need_tk ppu_lifter.py
        "$PY" "$TK/tools/ppu_lifter.py" "$M/game.elf" -j $JOBS --chunks 12 \
            --functions "$AN/game.functions.json" --hle-stubs "$AN/game.imports.json" \
            --insn-patches "$RES_ROOT/port/patches/game_v17.insn.json" --output "$RC" ;;
    spu)
        say "spu"; need_tk spu_lifter.py
        [ -f "$RES_ROOT/port/spu/captured/spu_recomp.c" ] ||
            { echo "no port/spu/captured (the policy module): run tools/rebuild.sh spu first" >&2; exit 1; }
        mkdir -p "$S"
        rm -rf "$S/captured"; cp -R "$RES_ROOT/port/spu/captured" "$S/captured"
        work=$(mktemp -d)
        "$PY" "$RES_ROOT/tools/extract_jobmods_elf.py" "$M/game.elf" --out "$work/jobmods"
        rm -rf "$S"/jobmod_*
        "$PY" "$TK/tools/lift_jobmods.py" --jobmods "$work/jobmods" --out "$S" \
            --register "$S/jobmods_register.c" --symbol-prefix resistance
        have_raw=1
        for pair in rawspu0:$RAW0 rawspu1:$RAW1; do
            name=${pair%%:*}; fp=${pair##*:}; bin=$RAWDIR/rawspu_${fp}_00060.bin
            if [ -f "$bin" ]; then
                "$PY" "$TK/tools/wrap_spu_elf.py" "$bin" --base 0x60 --entry 0x90 --out "$work/$name.elf"
                "$PY" "$TK/tools/spu_lifter.py" --auto-functions "$work/$name.elf" --base 0x60 \
                      --symbol-prefix "resistance_$name" --output "$S/$name"
            else
                have_raw=0; echo "  (no $bin yet: capture it, see the top of this script)"
            fi
        done
        rm -rf "$work"
        if (( have_raw )); then
            cp "$v17/spu_workloads.c" "$S/spu_workloads.c"
        else
            rm -rf "$S/rawspu0" "$S/rawspu1"
            sed -e '/spu_begin_image(2)/,/"resistance_rawspu_47C9CB480016D493");/d' -e '/rawspu/d' \
                "$v17/spu_workloads.c" > "$S/spu_workloads.c"
        fi ;;
    build)
        say "build"
        [ -f "$RC/ppu_recomp.h" ] || { echo "no lifted v1.7 code in $RC (steps analyse lift)" >&2; exit 1; }
        [ -f "$S/spu_workloads.c" ] || { echo "no $S/spu_workloads.c (step spu)" >&2; exit 1; }
        bp=${HOMEBREW_PREFIX:-$(brew --prefix 2>/dev/null)}
        cmake -S "$RES_ROOT/port" -B "$B" -G Ninja \
              -DPS3RECOMP_DIR="$TK" -DRECOMP_DIR="$RC" -DSPU_DIR="$S" \
              -DPS3RECOMP_FFMPEG_RUNTIME=ON -DPS3RECOMP_FFMPEG_INCLUDE_DIR="${bp:-/opt/homebrew}/include"
        cmake --build "$B" -j $JOBS ;;
    *)
        echo "unknown step '$s' (extract decrypt analyse lift spu build)" >&2; exit 2 ;;
    esac
done
say "done"
