#!/bin/sh
# Lift the two raw-SPU programs game.self starts after the level-1 intro movie.
#
#   tools/regen_rawspu.sh [DUMP_DIR]
#
# game.self streams both programs into raw SPU local store by proxy DMA (LS
# 0x60 onward, entry 0x90). They are not in any file on the disc in a form the
# offline tools can find, so they are captured at runtime: run game.self with
# SPU_RAW_DUMP_DIR=<dir> until the log says "[spu-raw] dumped -> ..." for both
# (tools/run.sh passes it through). DUMP_DIR defaults to $RES_OUT/rawspu.
#
#   rawspu_1894F66385AFD052_00060.bin  59,856 bytes  -> port/spu/rawspu0
#   rawspu_5A10969E466A2298_00060.bin  22,224 bytes  -> port/spu/rawspu1
#
# Each is wrapped with toolkit tools/wrap_spu_elf.py --base 0x60 --entry 0x90
# and lifted with tools/spu_lifter.py --auto-functions --base 0x60;
# port/spu/spu_workloads.c registers them as SPU images 2 and 3 under those
# fingerprints. The lifted C is translated game code: .gitignore'd, never
# commit it.
set -e
here=$(cd "$(dirname "$0")/.." && pwd)
TK=${PS3RECOMP_DIR:-$here/ps3recomp}
PY=${PY:-python3}
dumps=${1:-${RES_OUT:-$here/out}/rawspu}
work=${RES_OUT:-$here/out}/rawspu_lift

[ -f "$TK/tools/spu_lifter.py" ] || { echo "no toolkit at $TK (run tools/setup_toolkit.sh)" >&2; exit 1; }
mkdir -p "$work"

for pair in rawspu0:1894F66385AFD052 rawspu1:5A10969E466A2298; do
    name=${pair%%:*}; fp=${pair##*:}
    bin="$dumps/rawspu_${fp}_00060.bin"
    [ -f "$bin" ] || { echo "no $bin: capture it with SPU_RAW_DUMP_DIR=$dumps (see above)" >&2; exit 1; }
    "$PY" "$TK/tools/wrap_spu_elf.py" "$bin" --base 0x60 --entry 0x90 --out "$work/$name.elf"
    "$PY" "$TK/tools/spu_lifter.py" --auto-functions "$work/$name.elf" --base 0x60 \
          --symbol-prefix "resistance_$name" --output "$here/port/spu/$name"
    echo "lifted $fp -> port/spu/$name"
done
echo "re-run cmake so the new sources are picked up"
