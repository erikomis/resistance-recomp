# Créditos

🌐 [English](CREDITS.md) · **Português (BR)**

Este port só existe por causa do trabalho abaixo. Obrigado a todos.

## Toolkit

- **[ps3recomp](https://github.com/sp00nznet/ps3recomp)** — Ned Heller
  ([@sp00nznet](https://github.com/sp00nznet)) e
  [colaboradores](https://github.com/sp00nznet/ps3recomp/blob/master/CONTRIBUTORS.md).
  O runtime, as bibliotecas HLE, os tradutores de PPU e SPU, o backend Metal, o
  cellVdec com FFmpeg e o template de projeto de onde partem o `main.cpp`, o
  `stubs.cpp`, o `CMakeLists.txt` e o `config.toml` deste port. O port compila
  contra a v0.17.0 (`a6d76c8`). MIT.

## O port que serve de modelo

- **[infamous2-recomp](https://github.com/erikomis/infamous2-recomp)** — o port
  do inFAMOUS 2 para macOS, do mesmo autor, sobre o mesmo toolkit. De lá vêm a
  organização do repositório, a abordagem de patch do toolkit
  (`tools/setup_toolkit.sh`), os scripts de execução e captura (o
  `tools/ppm2png.py` foi copiado de lá) e a correção de rodar o guest fora da
  thread principal do macOS. Ele, por sua vez, se apoia no fork do inFAMOUS 1
  do ps3recomp feito por [NoahBPeterson](https://github.com/NoahBPeterson/ps3recomp).

## Ferramentas

- [RPCS3](https://github.com/RPCS3/rpcs3) — descriptografia do `EBOOT.BIN` e do
  `game.self` do próprio jogador (`rpcs3 --decrypt`), e a referência de
  comportamento do toolkit.
- [FFmpeg](https://ffmpeg.org/) (libavcodec, libavutil; LGPL/GPL, carregado em
  runtime a partir da instalação do Homebrew do próprio jogador, não
  distribuído aqui) — decodificação de vídeo.
- [SDL2](https://www.libsdl.org/), [glslang](https://github.com/KhronosGroup/glslang),
  [SPIRV-Cross](https://github.com/KhronosGroup/SPIRV-Cross),
  [SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools), CMake, Ninja.
- [SMAA](https://github.com/iryoku/smaa) — Jorge Jimenez, Jose I. Echevarria,
  Belen Masia, Fernando Navarro e Diego Gutierrez: o antisserrilhamento do
  `PS3_AA=smaa` (`third_party/smaa` no patch do toolkit, com o arquivo de
  licença). MIT.
- A [PS3 Developer Wiki](https://www.psdevwiki.com/ps3/).

## Jogo

Resistance: Fall of Man © 2006 Sony Interactive Entertainment LLC,
desenvolvido pela Insomniac Games. Este projeto não é afiliado nem endossado
pela Sony ou pela Insomniac, e não contém dados do jogo nem do firmware. Para
usá-lo você precisa de uma cópia legítima do jogo (o disco `BCUS98107`).
