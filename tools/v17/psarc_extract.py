#!/usr/bin/env python3
"""Extract files from a PSARC archive (PSAR v1.x, zlib blocks).

    psarc_extract.py <archive.psarc> <out_dir> [substring ...]

Writes every file whose archive path contains one of the substrings (all of
them when none is given) under <out_dir>, keeping the archive's paths, and
prints path, size and MD5 of each. Used for the v1.7 update: patch_06.psarc
holds /data/new_game.self.
"""
import hashlib, os, struct, sys, zlib


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    path, outdir, want = sys.argv[1], sys.argv[2], sys.argv[3:]
    with open(path, 'rb') as f:
        h = f.read(32)
        if h[:4] != b'PSAR':
            sys.exit(f'{path}: not a PSARC archive')
        if h[8:12] != b'zlib':
            sys.exit(f'{path}: compression {h[8:12]!r} not supported')
        toc_len, entry_sz, nent, blk, _flags = struct.unpack('>IIIII', h[12:32])
        f.seek(0)
        toc = f.read(toc_len)
        ents = []
        for i in range(nent):
            e = toc[32 + i * entry_sz: 32 + (i + 1) * entry_sz]
            ents.append((struct.unpack('>I', e[16:20])[0],
                         int.from_bytes(e[20:25], 'big'), int.from_bytes(e[25:30], 'big')))
        bt = 32 + nent * entry_sz
        bsz = 2 if blk <= 0x10000 else 3 if blk <= 0x1000000 else 4
        blocks = [int.from_bytes(toc[bt + i * bsz: bt + (i + 1) * bsz], 'big')
                  for i in range((toc_len - bt) // bsz)]

        def read(idx, ln, off):
            f.seek(off)
            out = bytearray()
            b = idx
            while len(out) < ln:
                z = blocks[b] if b < len(blocks) else 0
                data = f.read(blk if z == 0 else z)
                if z and data[:1] == b'\x78':
                    try:
                        data = zlib.decompress(data)
                    except zlib.error:
                        pass
                out += data
                b += 1
            return bytes(out[:ln])

        names = [n.decode('latin1') for n in read(*ents[0]).replace(b'\r', b'').split(b'\n') if n]
        if len(names) != len(ents) - 1:
            sys.exit(f'{path}: manifest lists {len(names)} names for {len(ents) - 1} files')
        n = 0
        base = os.path.realpath(outdir)
        for e, name in zip(ents[1:], names):
            if want and not any(w in name for w in want):
                continue
            # An entry name comes from the archive: keep it inside outdir.
            rel = name.lstrip('/')
            op = os.path.realpath(os.path.join(base, rel))
            if not rel or '..' in rel.split('/') or os.path.commonpath([base, op]) != base:
                sys.exit(f'{path}: entry {name!r} escapes the output directory')
            data = read(*e)
            os.makedirs(os.path.dirname(op), exist_ok=True)
            with open(op, 'wb') as o:
                o.write(data)
            print(f'{name} {len(data)} md5 {hashlib.md5(data).hexdigest()}')
            n += 1
        if want and not n:
            sys.exit(f'{path}: nothing matches {want}')


if __name__ == '__main__':
    main()
