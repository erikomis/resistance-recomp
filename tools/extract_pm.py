#!/usr/bin/env python3
"""Extract Resistance's SPURS job-manager policy module from the player's own
decrypted executable.

    tools/extract_pm.py game/BCUS98107/EBOOT.elf -o out/spu_pm/pm.bin
    tools/extract_pm.py spujob_4E4076E700C6AFF3_11136.bin -o out/spu_pm/pm.bin

The title hands this blob (11,136 bytes) to cellSpursAddWorkload. It is the
same bytes in both executables: guest 0x10191A80 in EBOOT.BIN and 0x10176E80
in game.self. The toolkit dispatches lifted SPU code by an FNV-1a 64 hash of
the image (spu_workload_fingerprint), so the result is checked against the
fingerprint port/spu/spu_workloads.c registers, 0x4E4076E700C6AFF3. A
different release of the game (or a bad decryption) fails here instead of
building a port whose policy module never matches.

The same bytes can be captured at runtime instead: run the port with
SPU_DUMP_MISS=<dir> and no lifted PM, and cellSpurs writes
<dir>/spujob_4E4076E700C6AFF3_11136.bin. Either file feeds
tools/regen_spu_pm.sh.
"""
import argparse
import struct
import sys

PM_SIZE = 11136
PM_FP = 0x4E4076E700C6AFF3
KNOWN_VA = (0x10191A80, 0x10176E80)   # EBOOT.BIN, game.self


def fnv1a64(data):
    h = 1469598103934665603
    for b in data:
        h ^= b
        h = (h * 1099511628211) & 0xFFFFFFFFFFFFFFFF
    return h


def load_segments(d):
    if d[:4] != b'\x7fELF' or d[4] != 2 or d[5] != 2:
        sys.exit('not a 64-bit big-endian ELF (decrypt EBOOT.BIN / game.self first)')
    phoff, = struct.unpack_from('>Q', d, 0x20)
    phentsize, phnum = struct.unpack_from('>HH', d, 0x36)
    segs = []
    for i in range(phnum):
        p_type, _flags, off, va, _pa, filesz, _memsz, _align = \
            struct.unpack_from('>IIQQQQQQ', d, phoff + i * phentsize)
        if p_type == 1 and filesz:                       # PT_LOAD
            segs.append((va, off, filesz))
    return segs


def read_va(d, segs, va, n):
    for sva, off, size in segs:
        if sva <= va and va + n <= sva + size:
            return d[off + va - sva: off + va - sva + n]
    return None


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('elf', help='decrypted EBOOT.elf or game.elf, or a captured spujob_*.bin')
    ap.add_argument('-o', '--out', required=True, help='raw policy-module bytes')
    ap.add_argument('--va', type=lambda s: int(s, 0), action='append',
                    help='guest address to try (default: the two known ones)')
    a = ap.parse_args()

    d = open(a.elf, 'rb').read()
    if d[:4] != b'\x7fELF':
        # A blob captured with SPU_DUMP_MISS: check it and pass it through.
        if len(d) != PM_SIZE or fnv1a64(d) != PM_FP:
            sys.exit('%s: %u bytes, fp 0x%016X -- not the policy module (fp 0x%016X, %u bytes)'
                     % (a.elf, len(d), fnv1a64(d), PM_FP, PM_SIZE))
        open(a.out, 'wb').write(d)
        print('captured policy module ok, fp 0x%016X -> %s' % (PM_FP, a.out))
        return 0
    segs = load_segments(d)
    for va in (a.va or KNOWN_VA):
        blob = read_va(d, segs, va, PM_SIZE)
        if blob is None:
            continue
        fp = fnv1a64(blob)
        if fp == PM_FP:
            open(a.out, 'wb').write(blob)
            print('policy module at 0x%08X, %u bytes, fp 0x%016X -> %s'
                  % (va, PM_SIZE, fp, a.out))
            return 0
        print('0x%08X: fp 0x%016X does not match' % (va, fp), file=sys.stderr)
    sys.exit('policy module (fp 0x%016X) not found in %s -- is this BCUS98107 v01.00?'
             % (PM_FP, a.elf))


if __name__ == '__main__':
    sys.exit(main())
