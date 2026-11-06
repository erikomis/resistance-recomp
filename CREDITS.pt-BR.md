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

## Correções do toolkit vindas de pull requests do upstream

O patch do toolkit traz estes pull requests do ps3recomp (alguns adaptados à
v0.17.0 ou a este port), com os autores mantidos nos cabeçalhos do patch:

- **undecuple** — a área de licença do console, América do Norte (corrigiu o
  texto sobreposto das telas legais; [#429](https://github.com/sp00nznet/ps3recomp/pull/429)); XInput carregado em tempo de
  execução no Windows ([#494](https://github.com/sp00nznet/ps3recomp/pull/494)).
- **n0madic** ([@n0madic](https://github.com/n0madic)) — os bits de desvio dos
  vertex programs ([#485](https://github.com/sp00nznet/ps3recomp/pull/485)); `WPOS` em render targets com antisserrilhamento
  ([#446](https://github.com/sp00nznet/ps3recomp/pull/446)).
- **Will Tekulve** ([@tekulvw](https://github.com/tekulvw)) — a condição de
  corrida da tabela de fds ([#430](https://github.com/sp00nznet/ps3recomp/pull/430)); decodificação rápida de A8R8G8B8 linear
  ([#432](https://github.com/sp00nznet/ps3recomp/pull/432)); a memória dos slots de textura ([#438](https://github.com/sp00nznet/ps3recomp/pull/438)); o decrementador da
  SPU no relógio bruto ([#451](https://github.com/sp00nznet/ps3recomp/pull/451)); ceder a vez em DMA de SPU repetido
  ([#448](https://github.com/sp00nznet/ps3recomp/pull/448)); pequenas economias de DMA de SPU ([#453](https://github.com/sp00nznet/ps3recomp/pull/453), [#454](https://github.com/sp00nznet/ps3recomp/pull/454), [#455](https://github.com/sp00nznet/ps3recomp/pull/455)).
- **Heitor Mendonça** — reposicionar o arquivo quando o fd alterna entre
  leitura e escrita ([#456](https://github.com/sp00nznet/ps3recomp/pull/456)).
- **Ryan Bagas** — a nova checagem da reserva no `stwcx.` ([#522](https://github.com/sp00nznet/ps3recomp/pull/522)).
- **Noah Peterson** ([@NoahBPeterson](https://github.com/NoahBPeterson)) — o
  ritmo em tempo real do mixer de áudio POSIX (do trabalho no inFAMOUS 2).
- **erikomis** — `getenv` consultado uma vez por ponto de chamada ([#515](https://github.com/sp00nznet/ps3recomp/pull/515));
  volume e saída de áudio; o giro opcional do `usleep`.

## Patches do jogo

- As opções de 60 fps e de campo de visão (`port/patches.cpp`,
  `port/patches/*.insn.json`) são os patches da comunidade no `patch.yml` do
  RPCS3 para este jogo, trazidos para a recompilação estática: **Whatcookie** e
  **FlexBy** ("Unlock FPS" do `game.self` 01.00 e da atualização),
  **illusion** ("Extended FOV") e **Score** ("60 FPS", menus e multiplayer).

## O port que serve de modelo

- **[infamous2-recomp](https://github.com/erikomis/infamous2-recomp)** — o port
  do inFAMOUS 2 para macOS, do mesmo autor, sobre o mesmo toolkit. De lá vêm a
  organização do repositório, a abordagem de patch do toolkit
  (`tools/setup_toolkit.sh`), os scripts de execução e captura (o
  `tools/ppm2png.py` foi copiado de lá), o descriptografador de SELF
  (`tools/decrypt_self.py`, cuja estrutura segue a do
  [sp00nznet/twistedmetal](https://github.com/sp00nznet/twistedmetal), MIT)
  e a correção de rodar o guest fora da
  thread principal do macOS. Ele, por sua vez, se apoia no fork do inFAMOUS 1
  do ps3recomp feito por [NoahBPeterson](https://github.com/NoahBPeterson/ps3recomp).

## Outro port do mesmo jogo

- **[Resistance: Fall of Man PC port](https://github.com/Psyykosis/Resistance-Fall-of-Man-Recomp)**
  — o port do mesmo jogo para Windows / Direct3D 12 feito por Psyykosis, sobre
  o mesmo commit do toolkit (MIT). Estas correções vêm dele, adaptadas a este
  port: o alpha test sobre o alpha que o buffer de cor guarda (a vegetação), o
  registrador de destino do `ARL` no programa de vértice, um `stwcx.` que grava
  o valor que já estava lá não derruba mais as reservas das SPUs, o "acordar"
  do canal das SPUs sob a trava de quem espera e a espera curta antes de uma
  SPU dormir, o `cellSpursReadyCountStore` sem trava, os saltos indiretos das
  SPUs alinhados à palavra, o código SPU traduzido sem as sondas de depuração
  da local store, e a corrida no layout das caixas de som (o centro/LFE
  variando na música do menu).

## Ferramentas

- [RPCS3](https://github.com/RPCS3/rpcs3) — a tabela pública de chaves dele
  (`rpcs3/Crypto/key_vault.cpp`), que o `tools/fetch_keys.py` baixa para
  descriptografar o `EBOOT.BIN` e o `game.self` do próprio jogador; o
  `rpcs3 --decrypt` como alternativa; e a referência de comportamento do
  toolkit.
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
