#!/usr/bin/env python3
"""Extract the raw-SPU programs linked into game.self (or EBOOT.BIN).

Resistance streams two SPU programs into raw SPUs by proxy DMA when a level
starts (spu_raw.c: LS 0x60 onward, entry 0x90). Both are linked into the
executable's data segment, and the PPU loader (v1.7: 0xD7C84 sets a program
up, 0xD79E4 DMAs it in 16 KB GETs) is only ever handed those two, as constant
{pointer, size} pairs in the GOT -- no level loads anything else. The image
format is the WWS job modules' cousin:

    +0x00  4 x ila          (executed: LS base .. base+0x10)
    +0x10  entry            (LS address, 0x90)
    +0x14  LS end           (image + bss)
    +0x18  stack/heap size
    +0x1C  entry again      (the loader puts the image at entry - 0x30)
    +0x20  parameter area   (LS address - entry; -0x91 = none)
    +0x24  parameter size
    +0x28  0xDEADDEAD 0xDEADDEAD

The parameter area is where the PPU writes, before every load, the pointers
and per-level values the program runs with (v1.7: 0x515E8 fills 0x48 bytes,
among them heap pointers and its own raw SPU's MMIO window). So the bytes a
raw SPU ends up with differ with the level, the checkpoint and the SPU number,
and spu_raw.c's plain fingerprint of the GET run is not stable. The port
registers each program with that area masked out
(spu_workload_register_img_masked); in the ELF the area is zero, so the
fingerprint to register is simply that of the bytes cut out here.

Output: <out>/rawspu_<FP>_<base:05X>.bin (the naming SPU_RAW_DUMP_DIR uses, so a
runtime capture and an extraction are interchangeable), the parameter area
zeroed, and with --manifest a JSON list of
{fp, ls_base, entry, size, var_off, var_len, vaddr}. var_off is relative to
the image start (= LS base), i.e. what spu_workload_register_img_masked takes.

Usage: extract_rawspu_elf.py game/BCUS98107/modules_v17/game.elf --out DIR
"""
import argparse, json, os, struct

MAGIC = b"\xDE\xAD\xDE\xAD\xDE\xAD\xDE\xAD"


def fnv1a64(b):
    h = 1469598103934665603
    for x in b:
        h = ((h ^ x) * 1099511628211) & 0xFFFFFFFFFFFFFFFF
    return h


def load_segments(d):
    if d[:4] != b"\x7fELF" or d[4] != 2 or d[5] != 2:
        raise SystemExit("not a 64-bit big-endian ELF")
    phoff, = struct.unpack_from(">Q", d, 0x20)
    phentsize, phnum = struct.unpack_from(">HH", d, 0x36)
    segs = []
    for i in range(phnum):
        p_type, _, off, va, _, filesz, _, _ = struct.unpack_from(">IIQQQQQQ", d, phoff + i * phentsize)
        if p_type == 1 and filesz:
            segs.append((off, va, filesz))
    return segs


def off_to_va(segs, off):
    for o, va, sz in segs:
        if o <= off < o + sz:
            return va + off - o
    return None


def find_size(d, segs, va):
    """The size the loader is handed with this image: the word after the
    image's address in the GOT ({pointer, size} pairs)."""
    pat = struct.pack(">I", va)
    sizes = set()
    i = d.find(pat)
    while i >= 0:
        if i % 4 == 0 and off_to_va(segs, i) is not None:
            n, = struct.unpack_from(">I", d, i + 4)
            if 0x30 < n <= 0x40000:
                sizes.add(n)
        i = d.find(pat, i + 1)
    return sizes


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("elf")
    ap.add_argument("--out", required=True)
    ap.add_argument("--manifest", help="write the found programs as JSON here")
    args = ap.parse_args()

    d = open(args.elf, "rb").read()
    segs = load_segments(d)
    os.makedirs(args.out, exist_ok=True)
    found = []
    i = d.find(MAGIC)
    while i >= 0:
        h = i - 0x28
        i = d.find(MAGIC, i + 8)
        if h < 0:
            continue
        ilas = struct.unpack_from(">4I", d, h)
        entry, ls_end, _, entry2, poff, plen = struct.unpack_from(">IIIIiI", d, h + 0x10)
        if not all((w >> 24) in (0x42, 0x43) for w in ilas) or entry != entry2 or entry < 0x30:
            continue
        va = off_to_va(segs, h)
        if va is None:
            continue
        base = entry - 0x30
        sizes = find_size(d, segs, va)
        if len(sizes) != 1:
            raise SystemExit("image at 0x%X (va 0x%08X): %s size in the GOT"
                             % (h, va, "no" if not sizes else "ambiguous %s" % sorted(sizes)))
        size = sizes.pop()
        if base + size > ls_end or h + size > len(d):
            raise SystemExit("image at 0x%X: size 0x%X runs past its LS end 0x%X" % (h, size, ls_end))
        img = bytearray(d[h:h + size])
        var_off = entry + poff - base
        var_len = plen if (plen and var_off >= 0 and var_off + plen <= size) else 0
        if not var_len:
            var_off = 0
        img[var_off:var_off + var_len] = bytes(var_len)
        fp = fnv1a64(img)
        name = "rawspu_%016X_%05X.bin" % (fp, base)
        with open(os.path.join(args.out, name), "wb") as f:
            f.write(img)
        found.append(dict(fp="%016X" % fp, ls_base=base, entry=entry, size=size,
                          var_off=var_off, var_len=var_len, vaddr=va, file=name))
        print("%s  va 0x%08X  LS 0x%05X..0x%05X entry 0x%05X  parameter area %s"
              % (name, va, base, base + size, entry,
                 "image+0x%X, 0x%X bytes" % (var_off, var_len) if var_len else "none"))
    if not found:
        raise SystemExit("no raw-SPU program found in %s" % args.elf)
    if args.manifest:
        with open(args.manifest, "w") as f:
            json.dump(found, f, indent=1)


if __name__ == "__main__":
    main()
