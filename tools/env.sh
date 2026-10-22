# Shared settings for the run scripts in tools/. Sourced, not run.
# Every value can be overridden from the environment.
#
#   RES_ROOT      this repository (default: the parent of tools/)
#   RES_BUILD     build directories (default: $RES_ROOT/build)
#                 $RES_BUILD/BCUS98107/ResistanceRecomp       the EBOOT build
#                 $RES_BUILD/BCUS98107_game/ResistanceRecomp  the game.self build
#                 $RES_BUILD/BCUS98107/Resistance             the window (launcher)
#                 (a build from before the rename, still named MyGameRecomp,
#                 is used when it is the only one there; rebuild to update it)
#   RES_GAME      decrypted executables (default: $RES_ROOT/game/BCUS98107)
#                 $RES_GAME/EBOOT.elf, $RES_GAME/modules/game.elf
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
# where <repo> is the parent of the build directory. RES_SPAWN_BIN and
# RES_SPAWN_ELF override that.

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
