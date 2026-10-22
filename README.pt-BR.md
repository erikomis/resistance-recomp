# Resistance: Fall of Man — recompilação estática para PC (macOS)

🌐 [English](README.md) · **Português (BR)**

Port nativo do **Resistance: Fall of Man** (PS3, disco `BCUS98107`, v01.00)
para **macOS/arm64**, feito com recompilação estática: o código PowerPC e SPU
do jogo é traduzido para C/C++ e compilado como executáveis nativos, com
gráficos em Metal. Não é um emulador.

**Ele chega ao gameplay, e o capítulo 1 já foi jogado com controle.** O
front end roda das telas legais pela instalação da primeira execução, pela
tela de título e pelo menu principal até uma campanha nova. O executável da
campanha então toca o vídeo de abertura da fase 1 e carrega "The Gauntlet"
(York). Numa janela, com um DualSense, York roda em primeira pessoa com o
HUD: inimigos e aliados, tiros, morrer e renascer. Roda a 7–8 fps, os prédios
aparecem azul forte e o jogador pode acabar acima do mapa (veja
[Estado](#estado)).

Construído sobre o [ps3recomp](https://github.com/sp00nznet/ps3recomp),
seguindo o [port do inFAMOUS 2](https://github.com/erikomis/infamous2-recomp)
do mesmo autor. Veja [CREDITS.pt-BR.md](CREDITS.pt-BR.md).

> [!WARNING]
> **Trabalho em andamento, bem no começo.** O capítulo 1 dá para jogar, mas
> lento e com erros de renderização. Não há release nem app: você mesmo
> compila a partir do seu disco. Só macOS com Apple silicon, e é preciso um
> controle. O que falta está no [ROADMAP.pt-BR.md](ROADMAP.pt-BR.md).

> **Este repositório não contém dados do jogo, firmware, chaves da Sony nem
> nada traduzido a partir deles.** Cada um gera tudo isso localmente a partir
> da própria cópia legítima. Projeto de fã, sem afiliação com a Sony
> Interactive Entertainment ou a Insomniac Games.

## Estado

Em 2026-10-22 (capítulo 1 jogado com controle), cada item verificado em execuções num Mac com Apple silicon e Metal.

| | |
| --- | --- |
| Os dois executáveis traduzidos para C++ e compilados (`EBOOT.BIN`: 25.022 funções; `game.self`: 18.994) | ✅ |
| Boot, telas legais | ✅ (o texto de uma tela legal anterior se sobrepõe à seguinte) |
| Instalação no HDD da primeira execução ("initial setup", grava os arquivos de cache `*.done`) | ✅ |
| Tela de título, menu principal (Campanha / Cooperativo / Multiplayer / Opções) | ✅ |
| Perguntas de dificuldade da campanha e de inverter o eixo Y | ✅ |
| Saves e perfil (`quick.sav`, `PROFILE.SAV`) gravados e lidos | ✅ |
| Passagem do EBOOT para o `game.self` (`sys_game_process_exitspawn`) até o vídeo de abertura da fase 1 | ✅ |
| Vídeos (cellVdec pelo FFmpeg, desenhados como quad em modo imediato) | ✅ a abertura da fase 1 toca até o fim |
| SPUs raw (dois programas que o `game.self` inicia depois da abertura) | ✅ traduzidas e rodando |
| Tela de carregamento do capítulo 1 ("The Gauntlet", York) | ✅ até "Press the START button" |
| Gameplay | ✅ capítulo 1 (York) jogado numa janela com um DualSense: primeira pessoa, HUD, inimigos e aliados, tiros, morrer e renascer |
| Taxa de quadros | 🚧 7–8 fps em janela (frames de ~130–146 ms): a thread do RSX fica ~100% ocupada decodificando vértices na CPU (`dc_fetch` ~41%, `rsx_vertex_remap_build` ~13%) e o resto do jogo espera por ela; um cache de vértices entre frames (vindo do port do inFAMOUS 2) está em andamento |
| Prédios e telhados | 🚧 desenhados em azul forte chapado (em investigação com dumps por passada) |
| Sombras | 🚧 retângulos escuros/pretos no chão (ainda não investigado) |
| Posição do jogador | 🚧 depois de morrer e renascer a visão fica acima da cidade (vista do céu, do avesso); num run headless andar para a frente também terminou nos telhados (ainda não investigado) |
| Jobs de SPU (geometria, ordenação de draws, efeitos, áudio NextSynth) | ✅ os 15 módulos de job WWS traduzidos |
| Depois do vídeo de abertura | ✅ o travamento intermitente foi corrigido (pilha de callback compartilhada) |
| Controle | ✅ DualSense, DualShock ou Xbox pelo SDL2 |
| Teclado e mouse | ⏳ não no macOS (o fallback de teclado do toolkit é só para Windows): é preciso um controle |
| Linux e Windows | ⏳ sem suporte |
| Multiplayer online | ❌ fora do escopo (os servidores oficiais acabaram) |

Detalhes e próximos passos no [ROADMAP.pt-BR.md](ROADMAP.pt-BR.md).

## Como funciona

O Resistance traz dois executáveis PPU no disco. O `EBOOT.BIN` é o front end
(telas legais, instalação, menus, multiplayer); o `game.self` é a campanha. No
console, escolher a dificuldade da campanha faz o EBOOT chamar
`sys_game_process_exitspawn`, que o substitui pelo `game.self`. Aqui cada
executável é traduzido e compilado separadamente, e o port implementa o
exitspawn (`port/stubs.cpp`) como um `execv` do outro build, repassando o
caminho de boot e os argumentos do guest e colocando o bloco de dados do pai
em topo-da-pilha-principal − 0x1000, onde o `game.self` procura o bloco
`"ELFtastic!"`.

As correções que trouxeram o port até aqui:

- **Job manager:** o policy module do job manager SPURS do jogo (fingerprint
  `4E4076E700C6AFF3`, 11.136 bytes) é traduzido na base de local store 0xA00,
  onde o SPURS o carrega ([tools/regen_spu_pm.sh](tools/regen_spu_pm.sh)).
- **Guest fora da thread principal do macOS** (`port/main.cpp`): o AppKit só
  roda na thread principal e, com o guest nela, o pump de eventos do backend
  Metal travava. O guest roda na própria thread e a thread principal gira o
  run loop, como no port do inFAMOUS 2.
- **Pilha da thread principal** (patch do toolkit,
  `runtime/syscalls/sys_ppu_thread.c`): `sys_ppu_thread_get_stack_information`
  devolvia 0 + 0 para a thread principal, e o `game.self` ia procurar o bloco
  do exitspawn em 0xFFFFF000.
- **Vídeo:** o cellVdec do toolkit com o FFmpeg carregado em runtime
  (`-DPS3RECOMP_FFMPEG_RUNTIME=ON`, FFmpeg do Homebrew).

As mudanças no toolkit são um único patch sobre o ps3recomp v0.17.0,
[patches/ps3recomp-a6d76c8-resistance.patch](patches/ps3recomp-a6d76c8-resistance.patch):
a informação de pilha da thread principal, o contexto de kernel SPURS que o
policy module lê e alguns diagnósticos (`VDEC_PICLOG`, `PPU_STACKINFO_LOG`,
`SPURS_PM_R4_WORD0`, `SPURS_PM_TRACE`, `RSX_ENGINE_PASS_DUMP_TRIGGER`,
comandos de analógico no `PAD_FILE`, uma linha de log mais completa no
`AddWorkload`).

## Compilando a partir do código

### O que você precisa

- Um Mac com Apple silicon (macOS com Metal).
- O seu próprio **disco do Resistance: Fall of Man, `BCUS98107` v01.00**,
  copiado para uma imagem e montado (a pasta que contém `PS3_GAME/`).
- O [RPCS3](https://rpcs3.net) para descriptografar os dois executáveis (se a
  descriptografia falhar, instale antes o firmware do PS3 no RPCS3). ELFs
  descriptografados de outro jeito também servem: é só colocá-los onde está
  indicado abaixo.
- Xcode Command Line Tools e Homebrew:

```sh
brew install cmake ninja sdl2 glslang spirv-cross spirv-tools ffmpeg python@3.12
```

### Compilando

```sh
git clone https://github.com/erikomis/resistance-recomp.git
cd resistance-recomp
tools/setup_toolkit.sh                        # ps3recomp v0.17.0 + patch em ./ps3recomp
python3.12 -m venv .venv
.venv/bin/pip install -r ps3recomp/requirements.txt
PY=.venv/bin/python PS3_VFS_ROOT=/Volumes/<seu disco> tools/rebuild.sh
```

O `tools/rebuild.sh` roda estas etapas (ou só as que você indicar, por
exemplo `tools/rebuild.sh lift build`):

| Etapa | O que faz | Grava |
| --- | --- | --- |
| `toolkit` | `tools/setup_toolkit.sh` | `ps3recomp/` |
| `decrypt` | copia o `EBOOT.BIN` e o `game.self` do disco e roda `rpcs3 --decrypt` (`RPCS3=<caminho>` se ele não estiver em `/Applications`); pulada quando o `.elf` já existe; confere o SHA-256 do resultado com o dos executáveis v01.00 testados | `game/BCUS98107/EBOOT.elf`, `game/BCUS98107/modules/game.elf` |
| `analyse` | o `ppu_loader.py` do toolkit (funções, imports, TOC) | `analysis/BCUS98107/` |
| `lift` | o `ppu_lifter.py` do toolkit, 16 pedaços para o EBOOT e 12 para o `game.self` | `recompiled/BCUS98107/`, `recompiled/BCUS98107/game_module/` |
| `spu` | `tools/regen_spu_pm.sh`: o policy module do job manager, extraído do seu EBOOT, embrulhado e traduzido | `port/spu/captured/` |
| `build` | CMake + Ninja, Metal, FFmpeg em runtime | `build/BCUS98107/MyGameRecomp`, `build/BCUS98107_game/MyGameRecomp` |

Tudo na última coluna é gerado a partir da sua cópia e está no `.gitignore`.
Mantenha essa organização: quando o EBOOT passa para a campanha, o
`port/stubs.cpp` executa `build/BCUS98107_game/MyGameRecomp` com
`game/BCUS98107/modules/game.elf` ao lado do build do EBOOT
(`RES_SPAWN_BIN` / `RES_SPAWN_ELF` mudam isso).

#### O policy module de SPU

O policy module do job manager é código do jogo traduzido, então é gerado de
novo localmente e nunca vai para o repositório. É o blob de 11.136 bytes que o
jogo passa para o `cellSpursAddWorkload` (guest 0x10191A80 no EBOOT,
0x10176E80 no `game.self`), fingerprint `4E4076E700C6AFF3`. O
`tools/regen_spu_pm.sh` faz:

```sh
tools/extract_pm.py game/BCUS98107/EBOOT.elf -o out/spu_pm/pm.bin     # confere a fingerprint
python ps3recomp/tools/wrap_spu_elf.py out/spu_pm/pm.bin --base 0xA00 --entry 0xA00 --out out/spu_pm/pm.elf
python ps3recomp/tools/spu_lifter.py --auto-functions out/spu_pm/pm.elf --base 0xA00 \
       --symbol-prefix resistance_spu --output port/spu/captured
```

O mesmo blob também pode ser capturado em tempo de execução: rode sem o PM
traduzido e com `SPU_DUMP_MISS=<pasta>`, e passe o
`spujob_4E4076E700C6AFF3_11136.bin` resultante para o
`tools/regen_spu_pm.sh`. O `port/spu/spu_workloads.c` registra a entrada
traduzida (`resistance_spu_spu_func_00000A00`) com essa fingerprint como
imagem SPU 1.

#### Os programas das SPUs raw

Depois da abertura da fase 1, o `game.self` carrega dois programas em SPUs raw
por DMA proxy (local store a partir de 0x60, entrada em 0x90). Eles são
capturados num run e traduzidos na base 0x60:

```sh
SPU_RAW_DUMP_DIR=$PWD/out/rawspu tools/to_campaign.sh rawspu   # depois espere ~5 min por "[spu-raw] dumped -> ..." duas vezes
tools/regen_rawspu.sh out/rawspu                          # -> port/spu/rawspu0, port/spu/rawspu1
```

O `port/spu/spu_workloads.c` registra os dois como imagens SPU 2
(`1894F66385AFD052`) e 3 (`5A10969E466A2298`). Como o policy module, o C
traduzido é código do jogo e fica fora do git.

## Jogando

Monte a imagem do seu disco, conecte ou pareie um controle (DualSense,
DualShock ou Xbox; no macOS teclado e mouse ainda não funcionam) e:

```sh
tools/play.sh
```

Ele abre o front end numa janela; começar uma campanha passa para o build do
`game.self` na mesma janela. No **primeiro boot aperte START duas vezes**: na
tela legal e depois na pergunta do "initial setup" (a instalação no HDD, feita
uma vez só). Saves, o cache da instalação e o log (`last.log`) ficam em
`~/Library/Application Support/resistance-recomp/` (`RES_DATA` muda isso). O
título da janela mostra a taxa de quadros. Feche a janela, ou aperte Ctrl+C no
terminal, para sair. Se faltar um build, o `play.sh` avisa: rode o
`tools/rebuild.sh`.

## Jogando pelo terminal

Monte o seu disco e:

```sh
tools/start.sh menu                  # o EBOOT numa janela, em segundo plano
tools/press.sh menu legal 0x0008     # START na tela legal
tools/press.sh menu setup 0x0008     # START na pergunta da instalação da primeira execução
tools/snap.sh menu agora             # out/menu/agora.png
tools/stop.sh
```

O primeiro boot precisa de **dois START**: a tela legal e depois a pergunta do
"initial setup" (a instalação no HDD, feita uma vez só; o `start.sh` mantém a
pasta de execução, `out/game.cwd/`, entre execuções). Com a instalação feita,
o `tools/to_campaign.sh <nome>` navega o menu até uma campanha nova no Médio e
a passagem para o `game.self`.

| Script | O que faz |
| --- | --- |
| `tools/play.sh` | jogar numa janela com controle (veja [Jogando](#jogando)) |
| `tools/run.sh <nome> <segundos> [eboot\|game]` | execução com tempo, um PNG do frame a cada 10 s em `out/<nome>/`, log em `out/<nome>.log`; pasta de execução nova a menos que `KEEP=1` ou `CWD=<pasta>` |
| `tools/start.sh <nome> [eboot\|game]` | execução em segundo plano controlada pelo `PAD_FILE`; `out/game.cwd/` persistente |
| `tools/press.sh <nome> <tag> <máscara> [polls] [espera]` | aperta um botão, espera e salva o frame como `<tag>.png` |
| `tools/snap.sh <nome> <tag>` | salva o frame atual |
| `tools/stop.sh` | para a execução do `start.sh` (só ela) |
| `tools/to_campaign.sh <nome>` | menu → campanha nova no Médio → `game.self` |
| `tools/ppm2png.py in.ppm out.png` | dump de frame para PNG, sem dependências |

Máscaras de botão: START `0x0008`, CIMA `0x0010`, DIREITA `0x0020`, BAIXO
`0x0040`, ESQUERDA `0x0080`, TRIÂNGULO `0x1000`, CÍRCULO `0x2000`, X `0x4000`
(combine com OR para apertar vários).

### Variáveis de ambiente

Os scripts definem estas por você ([tools/env.sh](tools/env.sh)); para rodar
um build direto: `build/BCUS98107/MyGameRecomp game/BCUS98107/EBOOT.elf`.

| Variável | O que faz |
| --- | --- |
| `PS3_VFS_ROOT=<pasta>` | o disco montado, a pasta que contém `PS3_GAME/` (os scripts acham um `BCUS98107` montado em `/Volumes` quando ela não está definida) |
| `PS3_HDD0_ROOT`, `PS3_HDD1_ROOT` | pastas graváveis para o `/dev_hdd0` (cache da instalação, saves, perfil) e o `/dev_hdd1` |
| `PS3RECOMP_FFMPEG_AVCODEC`, `PS3RECOMP_FFMPEG_AVUTIL` | caminhos da `libavcodec` / `libavutil` (no macOS, por exemplo `/opt/homebrew/lib/libavcodec.dylib`); `PS3RECOMP_FFMPEG=0` desliga a decodificação de vídeo |
| `PAD_SCRIPT="<seg>:<máscara>,..."` | botões em tempos fixos, por exemplo `PAD_SCRIPT="40:0x0008,70:0x0008"` para os dois START do primeiro boot |
| `PAD_FILE=<arquivo>` | entrada em malha fechada: escreva `<máscara> [polls]` no arquivo e o controle aperta no próximo poll e depois esvazia o arquivo; `S <lx> <ly> <rx> <ry> <polls>` segura os analógicos em vez disso (0–255, 128 = centro) |
| `PS3RECOMP_METAL_FRAME_DUMP=<arquivo.ppm>` | regrava esse arquivo com o frame a cada 120 apresentações |
| `PS3RECOMP_METAL_HEADLESS=1` | sem janela (renderização fora da tela) |
| `PS3RECOMP_METAL_FPS=1` | taxa de quadros e tempos de frame no título da janela (e linhas `[RSX FPS]` no log) |
| `RSX_ENGINE_PASS_DUMP_TRIGGER=<arquivo>` | com o dump por passada do draw engine (`RSX_ENGINE_PASS_DUMP=<pasta>`), começa a capturar quando esse arquivo aparece, para um script fazer o dump do frame até onde levou o jogo |
| `SPURS_PM_TRACE=1` | registra os polls do policy module do job manager (`[spurs-pm]`; desligado por padrão, custa taxa de quadros) |
| `PS3_TITLE` | título da janela |
| `RES_SPAWN_BIN`, `RES_SPAWN_ELF` | o build e o ELF que a passagem do exitspawn executa (padrão: a pasta de build irmã e `game/BCUS98107/…`) |
| `RES_SPAWN_KEEP=1` | mantém o arquivo de dados do exitspawn (para rodar o `game.self` de novo à mão com `RES_SPAWN_DATA=<arquivo>`) |
| `RES_PEEK=<seg>:<end>[,*<end>...]` | a cada `<seg>`, imprime 32 bytes da memória do guest em cada endereço (`*` desreferencia antes) |
| `SPU_DUMP_MISS=<pasta>` | grava em `<pasta>` toda imagem SPU / policy module que não tem código traduzido |
| `VDEC_PICLOG=1`, `PPU_STACKINFO_LOG=1` | registra as primeiras imagens decodificadas / cada consulta de informação de pilha |

## Organização

| Caminho | O que é |
| --- | --- |
| [`port/`](port/) | O port: `main.cpp` (boot, thread do guest, bloco de dados do exitspawn), `stubs.cpp` (exitspawn, `RES_PEEK`), `CMakeLists.txt`, `config.toml`, `spu/spu_workloads.c` (registro das SPUs) |
| [`patches/`](patches/) | O patch do toolkit sobre o ps3recomp v0.17.0 |
| [`tools/`](tools/) | `setup_toolkit.sh`, `rebuild.sh`, `regen_spu_pm.sh`, `extract_pm.py`, `play.sh` e os scripts de execução |
| [`ROADMAP.pt-BR.md`](ROADMAP.pt-BR.md) | Estado e próximos passos |

## Licença

Código deste repositório: MIT ([LICENSE](LICENSE)). O toolkit é MIT pelos
seus autores. Resistance: Fall of Man © Sony Interactive Entertainment LLC,
desenvolvido pela Insomniac Games; este projeto não é afiliado à Sony nem à
Insomniac.
