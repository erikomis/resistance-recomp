# Credits

🌐 **English** · [Português (BR)](CREDITS.pt-BR.md)

This port only exists because of the work below. Thank you all.

## Toolkit

- **[ps3recomp](https://github.com/sp00nznet/ps3recomp)** — Ned Heller
  ([@sp00nznet](https://github.com/sp00nznet)) and
  [contributors](https://github.com/sp00nznet/ps3recomp/blob/master/CONTRIBUTORS.md).
  The runtime, the HLE libraries, the PPU and SPU lifters, the Metal backend,
  the FFmpeg-backed cellVdec, and the project template this port's
  `main.cpp`, `stubs.cpp`, `CMakeLists.txt` and `config.toml` start from. The
  port builds against v0.17.0 (`a6d76c8`). MIT.

## Toolkit fixes from upstream pull requests

The toolkit patch carries these ps3recomp pull requests (some adapted to
v0.17.0 or to this port), with their authors kept in the patch headers:

- **undecuple** — the console's licence area, North America (it fixed the
  overlapping legal-screen text; [#429](https://github.com/sp00nznet/ps3recomp/pull/429)); XInput loaded at run time on
  Windows ([#494](https://github.com/sp00nznet/ps3recomp/pull/494)).
- **n0madic** ([@n0madic](https://github.com/n0madic)) — vertex-program branch
  bits ([#485](https://github.com/sp00nznet/ps3recomp/pull/485)); `WPOS` on antialiased render targets ([#446](https://github.com/sp00nznet/ps3recomp/pull/446)).
- **Will Tekulve** ([@tekulvw](https://github.com/tekulvw)) — the fs fd-table
  race ([#430](https://github.com/sp00nznet/ps3recomp/pull/430)); linear A8R8G8B8 fast decode ([#432](https://github.com/sp00nznet/ps3recomp/pull/432)); the texture-slot
  memo ([#438](https://github.com/sp00nznet/ps3recomp/pull/438)); the SPU decrementer on the raw clock ([#451](https://github.com/sp00nznet/ps3recomp/pull/451)); the yield
  on repeated SPU DMA ([#448](https://github.com/sp00nznet/ps3recomp/pull/448)); small SPU DMA savings ([#453](https://github.com/sp00nznet/ps3recomp/pull/453), [#454](https://github.com/sp00nznet/ps3recomp/pull/454), [#455](https://github.com/sp00nznet/ps3recomp/pull/455)).
- **Heitor Mendonça** — repositioning a file when its fd switches between
  reading and writing ([#456](https://github.com/sp00nznet/ps3recomp/pull/456)).
- **Ryan Bagas** — the `stwcx.` reservation re-check ([#522](https://github.com/sp00nznet/ps3recomp/pull/522)).
- **Noah Peterson** ([@NoahBPeterson](https://github.com/NoahBPeterson)) —
  real-time pacing of the POSIX audio mixer (from the inFAMOUS 2 work).
- **erikomis** — `getenv` looked up once per call site ([#515](https://github.com/sp00nznet/ps3recomp/pull/515)); audio volume
  and output device; the opt-in `usleep` spin.

## Game patches

- The 60 fps and field-of-view options (`port/patches.cpp`,
  `port/patches/*.insn.json`) are the community patches in RPCS3's
  `patch.yml` for this game, carried over to the static recompilation:
  **Whatcookie** and **FlexBy** ("Unlock FPS" for `game.self` 01.00 and the
  update), **illusion** ("Extended FOV") and **Score** ("60 FPS", menus and
  multiplayer).

## The port it follows

- **[infamous2-recomp](https://github.com/erikomis/infamous2-recomp)** — the
  author's inFAMOUS 2 port to macOS on the same toolkit. Its repository
  layout, the toolkit-patch approach (`tools/setup_toolkit.sh`), the run and
  capture scripts (`tools/ppm2png.py` is copied from it), the SELF decryptor
  (`tools/decrypt_self.py`, whose layout follows
  [sp00nznet/twistedmetal](https://github.com/sp00nznet/twistedmetal)'s, MIT)
  and the fix for
  running the guest off the macOS main thread all come from there. It in turn
  stands on [NoahBPeterson](https://github.com/NoahBPeterson/ps3recomp)'s
  inFAMOUS 1 fork of ps3recomp.

## In-game menu

- The in-game menu (`libs/video/ps3_overlay.mm`, in the toolkit patch) comes
  from the author's [inFAMOUS 2 port](https://github.com/erikomis/infamous2-recomp),
  adapted to this port; it is drawn with
  **[Dear ImGui](https://github.com/ocornut/imgui)** v1.92.9b by Omar Cornut
  and contributors (MIT), vendored in `third_party/imgui/`.

## Another port of the same game

- **[Resistance: Fall of Man PC port](https://github.com/Psyykosis/Resistance-Fall-of-Man-Recomp)**
  — Psyykosis's Windows / Direct3D 12 port of the same game on the same
  toolkit commit (MIT). These fixes come from it, adapted to this port: the
  alpha test on the alpha the colour buffer stores (the foliage), the vertex
  program's `ARL` destination register, a `stwcx.` that stores the value
  already there no longer taking the SPUs' reservations, the SPU channel wake
  under the waiter's lock and the short spin before an SPU sleeps, the
  lock-free `cellSpursReadyCountStore`, word-aligned SPU indirect branches,
  the lifted SPU code without local-store debug probes, and the speaker-layout
  race (the menu music's varying centre/LFE).

## Tools

- [RPCS3](https://github.com/RPCS3/rpcs3) — its public key table
  (`rpcs3/Crypto/key_vault.cpp`), which `tools/fetch_keys.py` downloads to
  decrypt the player's own `EBOOT.BIN` and `game.self`; `rpcs3 --decrypt` as
  the fallback; and the behaviour reference for the toolkit.
- [FFmpeg](https://ffmpeg.org/) (libavcodec, libavutil; LGPL/GPL, loaded at
  runtime from the player's own Homebrew install, not shipped) — video decoding.
- [SDL2](https://www.libsdl.org/), [glslang](https://github.com/KhronosGroup/glslang),
  [SPIRV-Cross](https://github.com/KhronosGroup/SPIRV-Cross),
  [SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools), CMake, Ninja.
- [SMAA](https://github.com/iryoku/smaa) — Jorge Jimenez, Jose I. Echevarria,
  Belen Masia, Fernando Navarro and Diego Gutierrez: the anti-aliasing behind
  `PS3_AA=smaa` (`third_party/smaa` in the toolkit patch, with its licence
  file). MIT.
- The [PS3 Developer Wiki](https://www.psdevwiki.com/ps3/).

## Game

Resistance: Fall of Man © 2006 Sony Interactive Entertainment LLC, developed
by Insomniac Games. This project is not affiliated with or endorsed by Sony or
Insomniac, and contains no game or firmware data. To use it you need a
legitimate copy of the game (the `BCUS98107` disc).
