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

## The port it follows

- **[infamous2-recomp](https://github.com/erikomis/infamous2-recomp)** — the
  author's inFAMOUS 2 port to macOS on the same toolkit. Its repository
  layout, the toolkit-patch approach (`tools/setup_toolkit.sh`), the run and
  capture scripts (`tools/ppm2png.py` is copied from it) and the fix for
  running the guest off the macOS main thread all come from there. It in turn
  stands on [NoahBPeterson](https://github.com/NoahBPeterson/ps3recomp)'s
  inFAMOUS 1 fork of ps3recomp.

## Tools

- [RPCS3](https://github.com/RPCS3/rpcs3) — decryption of the player's own
  `EBOOT.BIN` and `game.self` (`rpcs3 --decrypt`), and the behaviour reference
  for the toolkit.
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
