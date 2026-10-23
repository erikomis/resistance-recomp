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
HUD: inimigos e aliados, tiros, morrer e renascer no checkpoint. A cidade
aparece com as cores certas a uns 30 fps, numa janela só dos menus até a
campanha. A campanha é a da última atualização do jogo, a v1.7, traduzida de
novo a partir do `patch_06`; os menus continuam os da 01.00 do disco (veja
[Estado](#estado)).

Construído sobre o [ps3recomp](https://github.com/sp00nznet/ps3recomp),
seguindo o [port do inFAMOUS 2](https://github.com/erikomis/infamous2-recomp)
do mesmo autor. Veja [CREDITS.pt-BR.md](CREDITS.pt-BR.md).

> [!WARNING]
> **Trabalho em andamento, bem no começo.** O capítulo 1 dá para jogar, ainda
> com erros de renderização em aberto. Não há release: você mesmo compila a
> partir do seu disco (o [launcher](#launcher) roda a compilação e o jogo
> por você). Só macOS com Apple silicon, e é preciso um controle. O que falta
> está no [ROADMAP.pt-BR.md](ROADMAP.pt-BR.md).

> **Este repositório não contém dados do jogo, firmware, chaves da Sony nem
> nada traduzido a partir deles.** Cada um gera tudo isso localmente a partir
> da própria cópia legítima. Projeto de fã, sem afiliação com a Sony
> Interactive Entertainment ou a Insomniac Games.

## Estado

Em 2026-10-22 (a campanha v1.7, o app launcher, sombras e fumaça corrigidas), cada item verificado em execuções num Mac com Apple silicon e Metal.

| | |
| --- | --- |
| Os dois executáveis traduzidos para C++ e compilados (`EBOOT.BIN`: 25.022 funções; `game.self`: 18.994), e o `game.self` da atualização v1.7 (19.706 funções traduzidas, contra 19.219 do da 01.00) | ✅ |
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
| Taxa de quadros | ✅ ~30 fps no gameplay: 32,5 fps com o pacote de desempenho do inFAMOUS 2 contra 6,0 fps com as chaves dele desligadas (A/B, mesmo build, primeiros 60 s de gameplay); 30,5 fps na cidade com todas as correções, inclusive a do renascimento |
| Cores do mundo (prédios, telhados, ruas) | ✅ corretas: o mundo azul era um wrap do offset de vértice que o RSX faz e o port não fazia (veja [Como funciona](#como-funciona)) |
| Sombras e fumaça | ✅ os retângulos escuros no chão e a fumaça de bordas duras sumiram no jogo (conferido visualmente) depois das correções de profundidade de uma comparação com o RPCS3: a comparação de profundidade das sombras estava invertida, as texturas de profundidade agora preenchem todas as lanes (partículas suaves, fumaça), os snapshots de profundidade são recortados à região que a textura indica |
| Renascimento | ✅ depois de morrer o jogador volta no chão, onde fica o checkpoint (antes ia parar no céu; um reinício de SPU raw que o runtime descartava, veja [Como funciona](#como-funciona)); verificado em 12 mortes |
| Janela | ✅ uma janela persistente dos menus até a campanha: o pequeno launcher `Resistance` é dono dela e o jogo desenha nela por IOSurfaces compartilhadas, então a passagem do menu para a campanha não a fecha mais (~0,35 ms de GPU por frame, sem cópia na CPU); o "guest FPS" do título conta os flips do jogo |
| Jobs de SPU (geometria, ordenação de draws, efeitos, áudio NextSynth) | ✅ os 15 módulos de job WWS traduzidos |
| Depois do vídeo de abertura | ✅ o travamento intermitente foi corrigido (pilha de callback compartilhada) |
| Controle | ✅ DualSense, DualShock ou Xbox pelo SDL2 (a entrada com o processo do jogo em segundo plano está ligada para o launcher; ainda não testada assim com um controle físico); vibração ainda não verificada |
| Teclado e mouse | ⏳ não no macOS (o fallback de teclado do toolkit é só para Windows): é preciso um controle |
| Atualização v1.7 do jogo | ✅ a campanha roda o `game.self` da atualização, traduzido de novo a partir do `patch_06`: menus → Nova campanha → a abertura de York, gameplay e renascimento verificados; os menus continuam o EBOOT 01.00 do disco (veja [A campanha v1.7](#a-campanha-v17)); vibração com um controle físico, uma sessão longa e uma medição de fps da v1.7 ainda não verificadas |
| App launcher | ✅ [`launcher/`](launcher/) (Tauri): disco, builds, recompilação, ajustes, saves, Jogar/Parar, em português do Brasil e inglês |
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
- **O mundo azul:** o RSX soma o `VERTEX_DATA_BASE_OFFSET` ao offset de cada
  array de vértices e faz o wrap da soma em 28 bits, como o RPCS3 faz e as
  engines da Insomniac esperam. O stream de iluminação do mundo, refeito a
  cada frame, fica em base + offset = 0x1Fxxxxxx, que é a memória local
  0x0Fxxxxxx; sem o wrap o port lia além da VRAM, pegava zeros e iluminava o
  mundo de azul (`rsx_dsp_resolve_vertex_offset`, patch do toolkit, com teste
  de regressão).
- **Velocidade:** um cache de vértices entre frames (5,9 → 14 fps), um build
  otimizado (o port vinha sendo compilado em -O0; agora o runtime é -O2 e o
  código traduzido -O1 em toda configuração menos Debug) e o trabalho de
  desempenho do port do inFAMOUS 2 (cache de vértices por stream, ring de
  staging do Metal, bind só do estado que mudou, memo de programas, cache de
  shaders MSL, relógio de frame de 1 ms com um present por flip): 32,5 fps no
  gameplay com ele contra 6,0 fps com as chaves dele desligadas, 30,5 fps na
  cidade com todas as correções.
- **Renascimento:** a cada recarga do checkpoint o jogo escreve
  `SPU_RunCntl`=0 na SPU raw 1 (programa `5A10969E466A2298`), traz o programa
  de volta para o local store por DMA proxy e escreve `SPU_RunCntl`=1. O
  runtime ignorava a parada enquanto a SPU estava parada esperando num canal e
  descartava o reinício, então o programa antigo continuava rodando sobre a
  imagem recarregada e o jogador ia parar no céu. Agora, como no
  `RawSPUThread.cpp` do RPCS3, a parada para de fato a SPU raw (respeitada nas
  leituras e esperas de canal e na espera da mailbox de saída; a parada
  espera a thread do host) e o reinício a começa do zero
  (`runtime/spu/spu_raw.c`, patch do toolkit). Num teste com 12 mortes a
  câmera voltou sempre à posição da primeira vida (0 de 666 amostras fora da
  faixa, contra 193 de 292 antes).
- **Uma janela:** o exitspawn é um `execv`, e uma janela some junto com a
  imagem do processo que a abriu. Um launcher pequeno, `Resistance`
  (`tools/host_window/host_window.m` no toolkit, alvo `Resistance` no
  `port/CMakeLists.txt`), é dono da janela e roda o `ResistanceRecomp` com
  `PS3RECOMP_HOST_SURFACES` apontando para um nome de bootstrap Mach; o backend
  Metal então renderiza na GPU em IOSurfaces que compartilha com o launcher e
  não abre janela nem ícone no Dock. A imagem que o exec traz mantém o pid e
  se conecta de novo. Fechar a janela encerra o jogo; se o launcher morre, o
  jogo sai em ~2 s; sem o launcher o backend abre a própria janela, como
  antes.
- **A campanha v1.7:** o `patch_06` da atualização traz um `game.self` novo.
  Ele é traduzido como o do disco e compilado como `build/BCUS98107_game_v17`;
  no exitspawn o `port/stubs.cpp` o executa no lugar do build 01.00 quando ele
  e o ELF dele existem. 13 dos 15 módulos de job WWS e os dois programas das
  SPUs raw mudaram e são traduzidos a partir do ELF v1.7; o policy module é
  o mesmo.

As mudanças no toolkit são um único patch sobre o ps3recomp v0.17.0,
[patches/ps3recomp-a6d76c8-resistance.patch](patches/ps3recomp-a6d76c8-resistance.patch):
a informação de pilha da thread principal, o contexto de kernel SPURS que o
policy module lê, o offset de vértice de 28 bits, o cache de vértices e o
resto do trabalho de desempenho, as correções de uma comparação com o RPCS3
(direção da comparação das sombras, snapshots de profundidade recortados e
liberados, profundidade replicada em todas as lanes, texturas comprimidas
nunca resolvidas a partir de render targets, threads PPU começando com
VSCR[NJ]=1, sem `getenv` a cada escrita em canal MFC da SPU), a parada e o
reinício das SPUs raw pelo `SPU_RunCntl`, a janela por IOSurface compartilhada
(`libs/video/rsx_host_surface.h`, `tools/host_window/host_window.m`; a
janela recolhe o jogo com um `waitpid` bloqueante, então sai quando o jogo
sai, que é do que o Parar do launcher depende; o "guest FPS" do título conta
cada flip), a passagem do estado da janela pelo exitspawn e diagnósticos (`VDEC_PICLOG`,
`PPU_STACKINFO_LOG`, `SPURS_PM_R4_WORD0`, `SPURS_PM_TRACE`,
`RSX_ENGINE_PASS_DUMP_TRIGGER`, `RSX_ENGINE_PASS_DUMP_DRAWS`, `RSX_VPC_LOG`,
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
| `build` | CMake + Ninja, Metal, FFmpeg em runtime; também o launcher da janela, `Resistance` | `build/BCUS98107/ResistanceRecomp`, `build/BCUS98107_game/ResistanceRecomp`, `build/BCUS98107/Resistance` |
| `v17` | só quando pedida: a campanha v1.7 a partir do seu `patch_06.psarc` (`tools/v17/regen_v17.sh`, veja [A campanha v1.7](#a-campanha-v17)); o `build` também a recompila depois de traduzida | `game/BCUS98107/modules_v17/`, `recompiled/BCUS98107/game_module_v17/`, `port/spu_v17/`, `build/BCUS98107_game_v17/` |
| `launcher` | só quando pedida: o app de desktop [launcher](#launcher) (`npm ci`, `tauri build`; Node 20+ e Rust) | `build/launcher/Resistance Launcher.app` |

Tudo na última coluna é gerado a partir da sua cópia e está no `.gitignore`.
Mantenha essa organização: quando o EBOOT passa para a campanha, o
`port/stubs.cpp` executa `build/BCUS98107_game/ResistanceRecomp` com
`game/BCUS98107/modules/game.elf` ao lado do build do EBOOT, ou
`build/BCUS98107_game_v17/ResistanceRecomp` com
`game/BCUS98107/modules_v17/game.elf` quando os dois existem
(`RES_GAME_VERSION=100` mantém a 01.00; `RES_SPAWN_BIN` / `RES_SPAWN_ELF`
mudam tudo isso).

O executável se chamava `MyGameRecomp` até 2026-10-22, e os builds feitos
antes disso eram compilados sem otimização: compile de novo. O
`tools/setup_toolkit.sh` nunca reseta um checkout com um patch mais antigo,
então apague o `ps3recomp/` antes e rode `tools/rebuild.sh toolkit build`.
Enquanto isso, os scripts ainda rodam um build antigo `MyGameRecomp` quando
ele é o único que existe.

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

#### A campanha v1.7

A última atualização do jogo, a v1.7, são sete arquivos,
`patch_01`…`patch_07.psarc`; o `patch_06` traz um executável de campanha
novo (`/data/new_game.self`). O port o roda. O ELF descriptografado dele
(SHA-256 `e1d75e6d4afe…144f`, o "game.self Updated" do RPCS3,
PPU-a08a187d…) é traduzido como o do disco (19.706 funções contra 19.219), e
13 dos 15 módulos de job WWS e os dois programas das SPUs raw mudaram e são
traduzidos a partir dele (fingerprints novas `6D03497950639952` e
`47C9CB480016D493`; o policy module é idêntico byte a byte). Ele importa
`cellSaveDataAutoSave2`/`AutoLoad2`, `_sys_process_atexitspawn` e
`cellPadSetActDirect` (vibração), que a 01.00 não importa.

Os arquivos da atualização são as atualizações oficiais e gratuitas do jogo
e não estão neste repositório. Se você tem o jogo, o
[PSRewired](https://psrewired.com) deixa ele baixar a própria atualização no
RPCS3; copie os `patch_01`…`patch_07.psarc` do
`dev_hdd0/game/BCUS98107/USRDIR/data` do RPCS3 para `update/` neste checkout
(ou aponte `RES_UPDATE_DIR` para eles) e:

```sh
tools/rebuild.sh v17                                  # tools/v17/regen_v17.sh
SPU_RAW_DUMP_DIR=$PWD/out/rawspu_v17 tools/play.sh    # uma vez: campanha nova, deixe o vídeo de abertura acabar, feche a janela
tools/v17/regen_v17.sh spu build                      # traduz os dois programas das SPUs raw recém-capturados
```

O `regen_v17.sh` extrai o `new_game.self` do `patch_06`
(`tools/v17/psarc_extract.py`), descriptografa com `rpcs3 --decrypt` para
`game/BCUS98107/modules_v17/game.elf` (conferindo o SHA-256), traduz para
`recompiled/BCUS98107/game_module_v17/`, traduz o código de SPU dele para
`port/spu_v17/` (o policy module é copiado de `port/spu/captured/`, então
rode antes a etapa `spu`) e compila `build/BCUS98107_game_v17/`. Os programas
das SPUs raw são capturados num run, como na 01.00; até serem traduzidos, a
campanha v1.7 para depois do vídeo de abertura.

Os menus continuam os da 01.00. O `EBOOT.BIN` da atualização só existe na
forma NPDRM (edição digital), e o EBOOT 01.00 do disco não roda com os dados
da atualização montados (com o `patch_04` ou o `patch_06` ele cai ao carregar
as fontes do menu; com os outros ele trava no exitspawn). Então **não instale
os arquivos da atualização na pasta de jogo**: o `tools/play.sh` e o launcher
se recusam a começar enquanto houver um `patch_NN.bin` no `USRDIR/data`
dela. A campanha não precisa deles: são dados do front end e do multiplayer,
e o `game.self` nunca monta os arquivos da atualização. No exitspawn o log
mostra `campaign = v1.7 game.self`; `RES_GAME_VERSION=100` joga a campanha
01.00.

## Jogando

Monte a imagem do seu disco, conecte ou pareie um controle (DualSense,
DualShock ou Xbox; no macOS teclado e mouse ainda não funcionam) e:

```sh
tools/play.sh
```

Ele abre o front end numa janela do launcher `Resistance` (o `play.sh` o
compila em segundos se um build anterior a ele não o tiver); começar uma
campanha passa para o build do `game.self` na mesma janela. O `play.sh` diz
qual campanha é essa: a v1.7 quando ela está compilada (e ele a compila
quando o código v1.7 já foi traduzido mas não compilado), senão a 01.00
(veja [A campanha v1.7](#a-campanha-v17)). Ele se recusa a começar enquanto
os `patch_NN.bin` da atualização v1.7 estiverem na pasta de jogo. No **primeiro boot aperte START duas vezes**: na
tela legal e depois na pergunta do "initial setup" (a instalação no HDD, feita
uma vez só). Saves, o cache da instalação e o log (`last.log`) ficam em
`~/Library/Application Support/resistance-recomp/` (`RES_DATA` muda isso). O
título da janela mostra a taxa de quadros. Feche a janela, ou aperte Ctrl+C no
terminal, para sair. Se faltar um build, o `play.sh` avisa: rode o
`tools/rebuild.sh`.

## Launcher

[`launcher/`](launcher/) é um app de desktop (Tauri) que faz o que o
`tools/play.sh` faz sem terminal, no molde do launcher do port do inFAMOUS 2:
ele acha os seus builds e o seu disco, monta a imagem do disco por você
(`hdiutil`, só leitura) e a lembra, guarda os ajustes, e o **Jogar** abre a
janela `Resistance` com o build do EBOOT, a pasta de jogo
(`PS3_HDD0_ROOT`/`PS3_HDD1_ROOT`, `last.log`), o FFmpeg e o `caffeinate`, como
o `play.sh`. O **Parar** encerra o jogo e a janela dele (a janela agora sai
quando o jogo sai). Ele não leva nada do jogo: a capa da tela inicial é lida do seu
disco montado.

- **Início**: Jogar/Parar, o que ainda impede o Jogar (sem build, sem disco,
  os `patch_*.bin` da atualização v1.7 na pasta de jogo, com os quais o EBOOT
  01.00 não roda; um clique os tira de lá), o aviso do primeiro boot (dois
  STARTs), ajustes rápidos, o último save, o status, um atalho Jogar na Mesa e
  Adicionar à Steam.
- **Gráficos**: tamanho da janela (o `--size` da janela), o ícone do seu
  disco no Dock, FXAA, nitidez, filtragem anisotrópica, gama forçado, perfis.
- **Desempenho**: o cache de vértices e a divisão por fluxo, ligar só o
  estado que mudou, o cache de shaders (com o tamanho e Limpar) e os shaders
  em segundo plano, o relógio de quadros, o contador de fps, manter o Mac
  acordado.
- **Jogo**: o disco (imagem ou pasta, montar/ejetar), a versão da campanha
  (**v1.7** quando o build dela existe, ou **01.00**: `RES_GAME_VERSION=100`),
  o idioma do console (`PS3_LANG`), os vídeos (caminhos do FFmpeg), a pasta de
  jogo e um gerenciador de saves (o perfil e o `quick.sav`, aqui e no RPCS3:
  backup antes de cada Jogar, restaurar, copiar nos dois sentidos,
  exportar/importar .zip).
- **Controles**: um teste do controle (no macOS o jogo precisa de controle).
- **Avançado**: Instalar / recompilar (requisitos com instalação pelo
  Homebrew, a pasta do port, o `tools/rebuild.sh` com as etapas escolhidas e
  barra de progresso), Atualizações (`git fetch` do checkout, "builds mais
  velhos que as fontes", o release mais novo do launcher), o visualizador do
  log, o modo de depuração (uma pasta por sessão com capturas, um resumo do
  log e um .zip para anexar numa issue) e o ambiente com a linha de comando
  equivalente.

Interface em português do Brasil e inglês. Compile com Node 20+ e Rust:

```sh
tools/rebuild.sh launcher        # -> build/launcher/Resistance Launcher.app
```

Detalhes, capturas de tela e os comandos de desenvolvimento estão em
[launcher/README.md](launcher/README.md) (em inglês).

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
um build direto: `build/BCUS98107/Resistance -- build/BCUS98107/ResistanceRecomp
game/BCUS98107/EBOOT.elf` (ou, sem o launcher, o `ResistanceRecomp` abre a
própria janela).

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
| `RSX_ENGINE_PASS_DUMP_DRAWS=1` | com o dump por passada, uma linha por draw em `draws.txt`, agora com o formato e os primeiros bytes de cada atributo de vértice |
| `RSX_VPC_LOG=<constante>[,<draw>]` | a cada 30 frames, registra uma constante do vertex program como estava naquele draw (padrão: draw 40) junto com o número de draws do frame; `c16` é a posição da câmera nos shaders do mundo do Resistance |
| `PS3RECOMP_ENG_VCACHE=0`, `PS3RECOMP_ENG_VCACHE_SPLIT=0` | desligam o cache de vértices entre frames / a separação por stream (streams de malhas estáticas guardados à parte do stream de iluminação de cada frame) |
| `PS3RECOMP_METAL_BIND_ALL=1` | refaz o bind de todo o estado do Metal em cada draw em vez de só o que mudou |
| `PS3_SHADER_CACHE=<pasta>` | guarda os shaders MSL traduzidos entre execuções; `PS3_SHADER_ASYNC=1` (opcional) monta pipelines novos numa fila em segundo plano e pula os draws deles até ficarem prontos |
| `PS3_FRAMECLOCK_US=<µs>` | o tick do laço de frame do host (padrão 1000; 4000 é o ritmo antigo) |
| `SPURS_PM_TRACE=1` | registra os polls do policy module do job manager (`[spurs-pm]`; desligado por padrão, custa taxa de quadros) |
| `PS3_TITLE` | título da janela |
| `RES_GAME_VERSION=100` | a campanha 01.00 mesmo quando o build v1.7 existe |
| `RES_UPDATE_DIR=<pasta>` | onde o `tools/v17/regen_v17.sh` acha o `patch_06.psarc` (padrão `update/`) |
| `RES_SPAWN_BIN`, `RES_SPAWN_ELF` | o build e o ELF que a passagem do exitspawn executa (padrão: a pasta de build irmã e `game/BCUS98107/…`) |
| `PS3RECOMP_HOST_SURFACES` | definida pelo launcher `Resistance`: o nome de bootstrap Mach das superfícies compartilhadas da janela; não é para definir à mão |
| `PS3RECOMP_WINDOW_STATE` | definida pelo port no exitspawn para a próxima janela abrir onde estava a anterior; não é para definir à mão |
| `RES_SPAWN_KEEP=1` | mantém o arquivo de dados do exitspawn (para rodar o `game.self` de novo à mão com `RES_SPAWN_DATA=<arquivo>`) |
| `RES_PEEK=<seg>:<end>[,*<end>...]` | a cada `<seg>`, imprime 32 bytes da memória do guest em cada endereço (`*` desreferencia antes) |
| `SPU_DUMP_MISS=<pasta>` | grava em `<pasta>` toda imagem SPU / policy module que não tem código traduzido |
| `VDEC_PICLOG=1`, `PPU_STACKINFO_LOG=1` | registra as primeiras imagens decodificadas / cada consulta de informação de pilha |

## Organização

| Caminho | O que é |
| --- | --- |
| [`port/`](port/) | O port: `main.cpp` (boot, thread do guest, bloco de dados do exitspawn), `stubs.cpp` (exitspawn e a passagem do estado da janela, `RES_PEEK`), `CMakeLists.txt` (com o alvo do launcher `Resistance`), `config.toml`, `spu/spu_workloads.c` (registro das SPUs); `spu_v17/` é gerado pelo `tools/v17/regen_v17.sh` |
| [`patches/`](patches/) | O patch do toolkit sobre o ps3recomp v0.17.0 |
| [`tools/`](tools/) | `setup_toolkit.sh`, `rebuild.sh`, `regen_spu_pm.sh`, `extract_pm.py`, `play.sh` e os scripts de execução; `v17/` (a campanha v1.7: `regen_v17.sh`, `psarc_extract.py`, `spu_workloads.c`) |
| [`launcher/`](launcher/) | O launcher de desktop (Tauri): ajustes, disco, saves, recompilação e Jogar |
| [`ROADMAP.pt-BR.md`](ROADMAP.pt-BR.md) | Estado e próximos passos |

## Licença

Código deste repositório: MIT ([LICENSE](LICENSE)). O toolkit é MIT pelos
seus autores. Resistance: Fall of Man © Sony Interactive Entertainment LLC,
desenvolvido pela Insomniac Games; este projeto não é afiliado à Sony nem à
Insomniac.
