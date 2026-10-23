# Shared settings for the run scripts in tools/. Sourced, not run.
# Every value can be overridden from the environment.
#
#   RES_ROOT      this repository (default: the parent of tools/)
#   RES_BUILD     build directories (default: $RES_ROOT/build)
#                 $RES_BUILD/BCUS98107/ResistanceRecomp       the EBOOT build
#                 $RES_BUILD/BCUS98107_game/ResistanceRecomp  the game.self build
#                 $RES_BUILD/BCUS98107_game_v17/ResistanceRecomp  the v1.7 game.self
#                 build (optional, tools/v17/regen_v17.sh)
#                 $RES_BUILD/BCUS98107/Resistance             the window (launcher)
#                 (a build from before the rename, still named MyGameRecomp,
#                 is used when it is the only one there; rebuild to update it)
#   RES_GAME      decrypted executables (default: $RES_ROOT/game/BCUS98107)
#                 $RES_GAME/EBOOT.elf, $RES_GAME/modules/game.elf,
#                 $RES_GAME/modules_v17/game.elf (the v1.7 game.self, optional)
#   RES_GAME_VERSION  100 keeps the 01.00 campaign when the v1.7 build exists
#   RES_OUT       logs, frames, run directories (default: $RES_ROOT/out)
#   PS3_VFS_ROOT  your mounted disc (the folder holding PS3_GAME/); found
#                 under /Volumes by its title ID when unset
#   PS3RECOMP_FFMPEG_AVCODEC / PS3RECOMP_FFMPEG_AVUTIL
#                 FFmpeg libraries for the video decoder (default: Homebrew's)
#   PY            Python 3 (default: python3)
#
# The layout matters for more than these scripts: when the EBOOT exitspawns
# game.self, port/stubs.cpp execs the sibling build directory
# ($RES_BUILD/BCUS98107_game) with <repo>/game/BCUS98107/modules/game.elf,
# where <repo> is the parent of the build directory; when
# $RES_BUILD/BCUS98107_game_v17 and <repo>/game/BCUS98107/modules_v17/game.elf
# both exist it runs those instead (the v1.7 campaign) unless
# RES_GAME_VERSION=100. RES_SPAWN_BIN and RES_SPAWN_ELF override all of that.

RES_ROOT=${RES_ROOT:-${here:h}}
RES_BUILD=${RES_BUILD:-$RES_ROOT/build}
RES_GAME=${RES_GAME:-$RES_ROOT/game/BCUS98107}
RES_OUT=${RES_OUT:-$RES_ROOT/out}
PY=${PY:-python3}; export PY
export RES_OUT; mkdir -p "$RES_OUT"

# res_target eboot|game -> $B (binary) and $E (ELF)
res_target() {
    case ${1:-eboot} in
        eboot) B=$RES_BUILD/BCUS98107/ResistanceRecomp;      E=$RES_GAME/EBOOT.elf ;;
        game)  B=$RES_BUILD/BCUS98107_game/ResistanceRecomp; E=$RES_GAME/modules/game.elf ;;
        *) echo "unknown target '$1' (eboot or game)" >&2; return 1 ;;
    esac
    # An old build, from before the executable was renamed.
    [ ! -x "$B" ] && [ -x "${B:h}/MyGameRecomp" ] && B=${B:h}/MyGameRecomp
    [ -x "$B" ] || { echo "no build at $B (tools/rebuild.sh)" >&2; return 1; }
    [ -r "$E" ] || { echo "no $E (decrypt your disc's executables, see README)" >&2; return 1; }
}

# res_v17 -> 0 when the campaign will be the v1.7 game.self: its ELF and
# build are there (the build is made here when the lifted v1.7 code is) and
# RES_GAME_VERSION is not 100. Sets $CAMPAIGN to v1.7 or 01.00.
res_v17() {
    CAMPAIGN=01.00
    [ "$RES_GAME_VERSION" = 100 ] && return 1
    [ -r "$RES_GAME/modules_v17/game.elf" ] || return 1
    if [ ! -x "$RES_BUILD/BCUS98107_game_v17/ResistanceRecomp" ]; then
        [ -f "$RES_ROOT/recompiled/BCUS98107/game_module_v17/ppu_recomp.h" ] || return 1
        local steps=(build)
        [ -f "$RES_ROOT/port/spu_v17/spu_workloads.c" ] || steps=(spu build)
        echo "building the v1.7 campaign (tools/v17/regen_v17.sh ${steps[*]})..."
        "$RES_ROOT/tools/v17/regen_v17.sh" "${steps[@]}" >&2 || {
            echo "could not build the v1.7 campaign: the 01.00 one runs" >&2; return 1; }
    fi
    CAMPAIGN=v1.7
}

# res_update_files <play folder> -> 0 when the v1.7 update's patch_NN.bin
# files are in its USRDIR/data. The 01.00 EBOOT cannot run with the update's
# archives mounted (patch_04/06: it crashes loading the menu fonts; the
# others: it hangs at the exitspawn), and it mounts a patch_NN.psarc only
# when patch_NN.bin is beside it, so no .bin may be there.
res_update_files() {
    local f=("$1"/dev_hdd0/game/BCUS98107/USRDIR/data/patch_*.bin(N))
    (( ${#f} ))
}

# res_launcher -> $L, the program that owns the game window (target Resistance
# of the EBOOT build). A build configured before the target existed gets it
# here, in seconds. Run as: "$L" -- "$B" "$E". Without it the game opens its
# own window, which an exitspawn closes and reopens.
res_launcher() {
    L=$RES_BUILD/BCUS98107/Resistance
    [ -x "$L" ] && return 0
    echo "building the game window ($L)..."
    cmake --build "$RES_BUILD/BCUS98107" --target Resistance >&2 && [ -x "$L" ] && return 0
    echo "could not build $L (tools/rebuild.sh build)" >&2
    return 1
}

# The disc: PS3_VFS_ROOT, or the first mounted volume holding BCUS98107.
if [ -z "$PS3_VFS_ROOT" ]; then
    for d in /Volumes/*(N/); do
        if [ -f "$d/PS3_GAME/USRDIR/game.self" ] && grep -q BCUS98107 "$d/PS3_GAME/PARAM.SFO" 2>/dev/null; then
            PS3_VFS_ROOT=$d; break
        fi
    done
fi
res_disc() {
    [ -n "$PS3_VFS_ROOT" ] && [ -d "$PS3_VFS_ROOT/PS3_GAME" ] && return 0
    echo "disc not found: mount your BCUS98107 disc image and set PS3_VFS_ROOT to the folder holding PS3_GAME/" >&2
    return 1
}
export PS3_VFS_ROOT

# FFmpeg for cellVdec (built with -DPS3RECOMP_FFMPEG_RUNTIME=ON).
if [ -z "$PS3RECOMP_FFMPEG_AVCODEC" ] || [ -z "$PS3RECOMP_FFMPEG_AVUTIL" ]; then
    brew_prefix=${HOMEBREW_PREFIX:-$(brew --prefix 2>/dev/null)}
    brew_prefix=${brew_prefix:-/opt/homebrew}
    PS3RECOMP_FFMPEG_AVCODEC=${PS3RECOMP_FFMPEG_AVCODEC:-$brew_prefix/lib/libavcodec.dylib}
    PS3RECOMP_FFMPEG_AVUTIL=${PS3RECOMP_FFMPEG_AVUTIL:-$brew_prefix/lib/libavutil.dylib}
fi
export PS3RECOMP_FFMPEG_AVCODEC PS3RECOMP_FFMPEG_AVUTIL
