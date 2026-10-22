#!/bin/sh
# Regenerate the lifted SPURS job-manager policy module from your own copy.
#
#   tools/regen_spu_pm.sh [SOURCE]
#
# SOURCE is your decrypted EBOOT.elf or game.elf (default:
# game/BCUS98107/EBOOT.elf), or a spujob_4E4076E700C6AFF3_11136.bin captured
# at runtime with SPU_DUMP_MISS=<dir>. Writes port/spu/captured/spu_recomp.{c,h},
# which port/spu/spu_workloads.c registers as SPU image 1. That C is
# translated game code: it is .gitignore'd and must never be committed.
#
# Steps (the same ones the port was built with):
#   1. tools/extract_pm.py: the 11,136-byte blob the title passes to
#      cellSpursAddWorkload (guest 0x10191A80 in EBOOT.BIN, 0x10176E80 in
#      game.self), checked against fingerprint 0x4E4076E700C6AFF3;
#   2. toolkit tools/wrap_spu_elf.py --base 0xA00 --entry 0xA00: the blob is
#      a SPURS policy module, loaded and entered at LS 0xA00;
#   3. toolkit tools/spu_lifter.py --auto-functions <elf> --base 0xA00
#      --symbol-prefix resistance_spu.
#
# Environment: PS3RECOMP_DIR (toolkit, default ./ps3recomp), PY (python3),
# RES_OUT (scratch, default ./out).
set -e
here=$(cd "$(dirname "$0")/.." && pwd)
TK=${PS3RECOMP_DIR:-$here/ps3recomp}
PY=${PY:-python3}
src=${1:-$here/game/BCUS98107/EBOOT.elf}
work=${RES_OUT:-$here/out}/spu_pm
dest=${SPU_PM_OUT:-$here/port/spu/captured}

[ -f "$TK/tools/spu_lifter.py" ] || { echo "no toolkit at $TK (run tools/setup_toolkit.sh)" >&2; exit 1; }
[ -f "$src" ] || { echo "no $src: decrypt your EBOOT.BIN first (see README)" >&2; exit 1; }
mkdir -p "$work" "$dest"

"$PY" "$here/tools/extract_pm.py" "$src" -o "$work/pm.bin"
"$PY" "$TK/tools/wrap_spu_elf.py" "$work/pm.bin" --base 0xA00 --entry 0xA00 --out "$work/pm.elf"
"$PY" "$TK/tools/spu_lifter.py" --auto-functions "$work/pm.elf" --base 0xA00 \
      --symbol-prefix resistance_spu --output "$dest"
echo "lifted policy module -> $dest (re-run cmake so it is picked up)"
