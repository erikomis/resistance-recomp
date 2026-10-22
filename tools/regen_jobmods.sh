#!/bin/sh
# Lift the WWS SPU job modules the game links into its executables.
#
#   tools/regen_jobmods.sh [ELF]
#
# ELF defaults to game/BCUS98107/modules/game.elf (EBOOT.elf carries the same
# 15 modules). Each module has a 0xC0DEC0DE header; 13 are linked at LS 0x4000
# and two (97b832367e51, b2ccb1ad289d: the audio jobs) at 0x7000, which the
# toolkit's tools/lift_jobmods.py reads from the header (+0x28) with this
# repository's toolkit patch. Writes port/spu/jobmod_*/spu_recomp.{c,h} and
# port/spu/jobmods_register.c (SPU images 200..214, matched by content
# signature when the job manager DMAs a module in). Translated game code:
# .gitignore'd, never commit it.
set -e
here=$(cd "$(dirname "$0")/.." && pwd)
TK=${PS3RECOMP_DIR:-$here/ps3recomp}
PY=${PY:-python3}
elf=${1:-$here/game/BCUS98107/modules/game.elf}
work=${RES_OUT:-$here/out}/jobmods

[ -f "$TK/tools/lift_jobmods.py" ] || { echo "no toolkit at $TK (run tools/setup_toolkit.sh)" >&2; exit 1; }
[ -f "$elf" ] || { echo "no $elf: decrypt your disc's executables first (see README)" >&2; exit 1; }
rm -rf "$work"; mkdir -p "$work"
"$PY" "$here/tools/extract_jobmods_elf.py" "$elf" --out "$work"
"$PY" "$TK/tools/lift_jobmods.py" --jobmods "$work" --out "$here/port/spu" \
      --register "$here/port/spu/jobmods_register.c" --symbol-prefix resistance
echo "lifted job modules -> port/spu (re-run cmake so they are picked up)"
