#!/bin/sh
# Lift the two raw-SPU programs game.self starts when a level loads.
#
#   tools/regen_rawspu.sh [GAME_ELF]
#
# game.self streams both programs into raw SPU local store by proxy DMA (LS
# 0x60 onward, entry 0x90). Both are linked into its data segment, and its PPU
# loader is only ever handed those two (constant {pointer, size} pairs in the
# GOT; EBOOT.BIN links the same two), so tools/extract_rawspu_elf.py cuts them
# out of the decrypted ELF (default game/BCUS98107/modules/game.elf):
#
#   rawspu_1894F66385AFD052_00060.bin  59,856 bytes  -> port/spu/rawspu0
#   rawspu_2B2DB6A5DE233A5F_00060.bin  22,224 bytes  -> port/spu/rawspu1
#
# The second one has a 0x50-byte parameter area the PPU fills in before every
# load (heap pointers, per-level values, its raw SPU's MMIO window), so its
# fingerprint as loaded changes with the level, the checkpoint and the SPU
# number (5A10969E466A2298 in one run); port/spu/spu_workloads.c registers it
# with that area masked (spu_workload_register_img_masked).
#
# Each is wrapped with toolkit tools/wrap_spu_elf.py --base 0x60 --entry 0x90
# and lifted with tools/spu_lifter.py --auto-functions --base 0x60;
# port/spu/spu_workloads.c registers them as SPU images 2 and 3. The lifted C
# is translated game code: .gitignore'd, never commit it.
set -e
here=$(cd "$(dirname "$0")/.." && pwd)
TK=${PS3RECOMP_DIR:-$here/ps3recomp}
PY=${PY:-python3}
elf=${1:-$here/game/BCUS98107/modules/game.elf}
work=${RES_OUT:-$here/out}/rawspu_lift

[ -f "$TK/tools/spu_lifter.py" ] || { echo "no toolkit at $TK (run tools/setup_toolkit.sh)" >&2; exit 1; }
[ -f "$elf" ] || { echo "no $elf (tools/rebuild.sh decrypt)" >&2; exit 1; }
rm -rf "$work"; mkdir -p "$work"
"$PY" -I "$here/tools/extract_rawspu_elf.py" "$elf" --out "$work/bin"
n=$(ls "$work/bin" | wc -l | tr -d ' ')
[ "$n" = 2 ] || { echo "$elf links $n raw-SPU programs, port/spu/spu_workloads.c registers 2" >&2; exit 1; }

for pair in rawspu0:1894F66385AFD052 rawspu1:2B2DB6A5DE233A5F; do
    name=${pair%%:*}; fp=${pair##*:}
    bin="$work/bin/rawspu_${fp}_00060.bin"
    [ -f "$bin" ] || { echo "no raw-SPU program $fp in $elf (not the tested 01.00 game.self?)" >&2; exit 1; }
    "$PY" "$TK/tools/wrap_spu_elf.py" "$bin" --base 0x60 --entry 0x90 --out "$work/$name.elf"
    rm -rf "$here/port/spu/$name"
    "$PY" "$TK/tools/spu_lifter.py" --auto-functions "$work/$name.elf" --base 0x60 \
          --symbol-prefix "resistance_$name" --output "$here/port/spu/$name"
    echo "lifted $fp -> port/spu/$name"
done
echo "re-run cmake so the new sources are picked up"
