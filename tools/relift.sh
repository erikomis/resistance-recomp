#!/bin/zsh
# Re-lift the PPU code of the EBOOT and/or the campaign with the run-time
# switchable instruction patches of port/patches/ (port/patches.cpp has the
# data half of each patch and the switches), then rebuild:
#
#   tools/relift.sh [eboot] [game] [v17]    (default: all three that exist)
#
#   eboot  EBOOT.elf            -> recompiled/BCUS98107                 (eboot.insn.json)
#   game   modules/game.elf     -> recompiled/BCUS98107/game_module     (game_100.insn.json)
#   v17    modules_v17/game.elf -> recompiled/BCUS98107/game_module_v17 (game_v17.insn.json)
#
# The arguments are those of tools/rebuild.sh lift and tools/v17/regen_v17.sh
# lift; with an empty patch list the output is byte-identical to a lift without
# --insn-patches. The lifter rewrites only the chunks whose text changes, so
# the rebuild recompiles one or two of them. Use it on a lift made before the
# patches existed (RES_UNLOCK_FPS / RES_MENU_60FPS then do nothing). Everything
# it writes is game-derived and .gitignore'd. NOBUILD=1 skips the rebuild.
set -e -o pipefail
here=${0:A:h}; source "$here/env.sh"
TK=${PS3RECOMP_DIR:-$RES_ROOT/ps3recomp}
AN=$RES_ROOT/analysis/BCUS98107
RC=$RES_ROOT/recompiled/BCUS98107
P=$RES_ROOT/port/patches
JOBS=${JOBS:-$(sysctl -n hw.logicalcpu 2>/dev/null || echo 4)}
if (( $# )); then which=("$@"); else which=(eboot game v17); fi
for w in $which; do
    case $w in
    eboot) elf=$RES_GAME/EBOOT.elf;            an=$AN;                 out=$RC;                 n=16; fn=EBOOT; spec=eboot;    b=BCUS98107 ;;
    game)  elf=$RES_GAME/modules/game.elf;     an=$AN/game_module;     out=$RC/game_module;     n=12; fn=game;  spec=game_100; b=BCUS98107_game ;;
    v17)   elf=$RES_GAME/modules_v17/game.elf; an=$AN/game_module_v17; out=$RC/game_module_v17; n=12; fn=game;  spec=game_v17; b=BCUS98107_game_v17 ;;
    *) echo "unknown '$w' (eboot game v17)" >&2; exit 2 ;;
    esac
    if [ ! -f "$an/$fn.functions.json" ]; then
        (( $# )) && { echo "no analysis for $w in $an (tools/rebuild.sh analyse)" >&2; exit 1; }
        continue
    fi
    print -P "%B==> lift $w%b"
    "$PY" "$TK/tools/ppu_lifter.py" "$elf" -j $JOBS --chunks $n \
        --functions "$an/$fn.functions.json" --hle-stubs "$an/$fn.imports.json" \
        --insn-patches "$P/$spec.insn.json" --output "$out"
    if [ -z "$NOBUILD" ] && [ -f "$RES_BUILD/$b/build.ninja" ]; then
        print -P "%B==> build $b%b"
        cmake --build "$RES_BUILD/$b" -j $JOBS
    fi
done
