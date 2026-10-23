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
aparece com as cores certas a uns 30 fps (uns 56 fps com o patch opcional de
60 fps), numa janela só dos menus até a campanha. A campanha é a da última atualização do jogo, a v1.7, traduzida de
novo a partir do `patch_06`; os menus continuam os da 01.00 do disco (veja
[Estado](#estado)).

Construído sobre o [ps3recomp](https://github.com/sp00nznet/ps3recomp),
seguindo o [port do inFAMOUS 2](https://github.com/erikomis/infamous2-recomp)
do mesmo autor. Veja [CREDITS.pt-BR.md](CREDITS.pt-BR.md).

> [!WARNING]
> **Trabalho em andamento, bem no começo.** O capítulo 1 dá para jogar, ainda
> com erros de renderização em aberto. Os [releases](#releases--download)
> trazem só o launcher: você mesmo compila o jogo a partir do seu disco (o
> [launcher](#launcher) roda a compilação e o jogo por você). Só macOS com Apple silicon. O controle é o jeito testado de jogar; teclado
> e mouse são novos e ainda precisam de um teste jogando. O que falta
> está no [ROADMAP.pt-BR.md](ROADMAP.pt-BR.md).

> **Este repositório não contém dados do jogo, firmware, chaves da Sony nem
> nada traduzido a partir deles.** Cada um gera tudo isso localmente a partir
> da própria cópia legítima. Projeto de fã, sem afiliação com a Sony
> Interactive Entertainment ou a Insomniac Games.

## Releases / Download

A [página de releases](https://github.com/erikomis/resistance-recomp/releases)
tem o [launcher](#launcher) para macOS com Apple silicon (um `.app` zipado e
um `.dmg`), o patch do toolkit e o código-fonte. **O jogo não vem junto**: os
executáveis do port são código convertido do jogo, então você os compila a
partir do seu próprio disco ([Compilando a partir do código](#compilando-a-partir-do-código)).
Os builds não são assinados: clique com o botão direito no app e escolha
Abrir na primeira vez.

Um **launcher para Windows x64** (`.exe` NSIS e `.msi`) é publicado como
**prévia**: ele compila e abre, monta uma ISO e roda o
`build\BCUS98107\ResistanceRecomp.exe`, mas o port em si nunca foi compilado
nem jogado no Windows, então trate-o como experimental e não testado (o
SmartScreen pode avisar; "Mais informações" > "Executar assim mesmo").
Detalhes em [launcher/README.md](launcher/README.md#windows-preview).

O release mais recente é a **v0.2.0** (2026-10-23, um pré-release): o
launcher para macOS e a prévia para Windows com os ajustes adicionados desde a
v0.1.0 (controles, modos de janela, filtros, pacotes de texturas, 60 fps e
campo de visão, áudio). Para compilar o launcher a partir do código:
`tools/rebuild.sh launcher`.

## Estado

Em 2026-10-23 (opções de 60 fps e de campo de visão; a sobreposição das telas legais corrigida; ritmo do áudio, volume e saída de áudio; 18 correções do toolkit vindas de pull requests do upstream; depois o Continuar a partir de um checkpoint, o som dos vídeos, a névoa e a água falsa da fase do rio e as sombras dos soldados distantes corrigidos), cada item ✅ verificado em execuções num Mac com Apple silicon e Metal; 🧪 = compilado e testado, ainda não verificado numa sessão de jogo.

| | |
| --- | --- |
| Os dois executáveis traduzidos para C++ e compilados (`EBOOT.BIN`: 25.022 funções; `game.self`: 18.994), e o `game.self` da atualização v1.7 (19.706 funções traduzidas, contra 19.219 do da 01.00) | ✅ |
| Boot, telas legais | ✅ logos → título "Press the START button", sem texto legal sobreposto (o runtime informava a área de licença de um console japonês; agora informa América do Norte, `PS3_LICENSE_AREA` a troca) |
| Instalação no HDD da primeira execução ("initial setup", grava os arquivos de cache `*.done`) | ✅ |
| Tela de título, menu principal (Campanha / Cooperativo / Multiplayer / Opções) | ✅ |
| Perguntas de dificuldade da campanha e de inverter o eixo Y | ✅ |
| Saves e perfil (`quick.sav`, `PROFILE.SAV`) gravados e lidos | ✅ |
| Passagem do EBOOT para o `game.self` (`sys_game_process_exitspawn`) até o vídeo de abertura da fase 1 | ✅ |
| Vídeos (cellVdec pelo FFmpeg, desenhados como quad em modo imediato) | ✅ a abertura da fase 1 toca até o fim, com som: o cellAdec decodifica as faixas AC3 5.1 pelo FFmpeg (era um stub que devolvia silêncio, então todo vídeo era mudo) |
| SPUs raw (dois programas que o `game.self` inicia quando uma fase carrega) | ✅ recortados do ELF (sem run de captura) e traduzidos; Continuar a partir de um checkpoint carrega e joga (travava: o programa tem uma área de parâmetros que muda com a fase e o checkpoint, agora mascarada na fingerprint dele) |
| Tela de carregamento do capítulo 1 ("The Gauntlet", York) | ✅ até "Press the START button" |
| Gameplay | ✅ capítulo 1 (York) jogado numa janela com um DualSense: primeira pessoa, HUD, inimigos e aliados, tiros, morrer e renascer |
| Taxa de quadros | ✅ a campanha chega ao limite do próprio jogo, ~30–33 fps (o Resistance roda a 30 fps no PS3); o tempo "ocupado" que resta na PPU é em boa parte o limitador de quadros do jogo. Desde 2026-10-22: o buffer de profundidade de 2560×720, que o jogo amostra como A8R8G8B8 a cada quadro, é resolvido na GPU (era uma leitura GPU→CPU e um reenvio de 7,4 MB por quadro): thread do RSX de 54% → 12% ocupada, reenvios de textura a cada 120 quadros de ~100 → 0; veja [Como funciona](#como-funciona) |
| 60 fps e campo de visão maior | ✅ opcionais, os patches da comunidade para o RPCS3 trazidos para o port (`port/patches.cpp`): `RES_UNLOCK_FPS=1` ~56 fps na campanha (01.00 e v1.7, medido com a máquina ociosa) na velocidade normal do jogo (o movimento de câmera da introdução leva 14,4–16,4 s a 33, 56 e 100 fps; andar ~6,3 unidades/s), `RES_MENU_60FPS=1` menus de 32 → 59 fps, `RES_FOV=<0.1..4>` (o do jogo é 1,0; o "Extended FOV" do RPCS3 é 0,7), `PS3_VBLANK_HZ=120` ~100 fps; morrer e renascer com o destravamento ligado ainda não foi exercitado |
| Áudio | ✅ o mixer do macOS segue o tempo real (alternava entre 2,4× e 0,95×; agora mistura um bloco a cada 5,333 ms pelo relógio e reduz 7.1 para estéreo sem clipping); o som dos vídeos (veja Vídeos); a música do menu às vezes tem conteúdo a mais no centro/LFE (+4–6 dB), que vem do sintetizador do jogo e varia de uma execução para outra; 🧪 `PS3_AUDIO_VOLUME=0..100`, `PS3_AUDIO_DEVICE=<nome>`, `PS3_AUDIO_MUTE=1` (também no launcher) |
| Cores do mundo (prédios, telhados, ruas) | ✅ corretas: o mundo azul era um wrap do offset de vértice que o RSX faz e o port não fazia (veja [Como funciona](#como-funciona)) |
| Sombras e fumaça | ✅ os retângulos escuros no chão e a fumaça de bordas duras sumiram no jogo (conferido visualmente) depois das correções de profundidade de uma comparação com o RPCS3: a comparação de profundidade das sombras estava invertida, as texturas de profundidade agora preenchem todas as lanes (partículas suaves, fumaça), os snapshots de profundidade são recortados à região que a textura indica; os soldados distantes não ficam mais sobre retângulos escuros (os shadow maps de profundidade de 16 bits em ordem swizzled agora são gravados de volta na memória do guest) |
| Água, névoa no chão e fumaça | ✅ a fase do rio mostra o leito seco do riacho com névoa suave em vez de um "reflexo" falso com listras verticais: o `WPOS` dos fragment shaders ignorava a origem embaixo à esquerda da janela, então os shaders de névoa e fumaça liam o buffer de profundidade de cabeça para baixo (verificado nas duas campanhas) |
| Folhagem | ⚠️ pontos escuros e bordas pretas nas folhas; ainda não confirmado se o original é igual |
| Renascimento | ✅ depois de morrer o jogador volta no chão, onde fica o checkpoint (antes ia parar no céu; um reinício de SPU raw que o runtime descartava, veja [Como funciona](#como-funciona)); verificado em 12 mortes |
| Janela | ✅ uma janela persistente dos menus até a campanha: o pequeno launcher `Resistance` é dono dela e o jogo desenha nela por IOSurfaces compartilhadas, então a passagem do menu para a campanha não a fecha mais (~0,35 ms de GPU por frame, sem cópia na CPU); o "guest FPS" do título conta os flips do jogo. Uma janela em tela cheia no próprio Space para de apresentar enquanto outro app está na frente (é o macOS, não um travamento): o log avisa ("window off screen", `hidden=1` no `[RSX FPS]`) e o quadro mais novo aparece na volta |
| Jobs de SPU (geometria, ordenação de draws, efeitos, áudio NextSynth) | ✅ os 15 módulos de job WWS traduzidos |
| Depois do vídeo de abertura | ✅ o travamento intermitente foi corrigido (pilha de callback compartilhada) |
| Controle | ✅ DualSense, DualShock ou Xbox pelo SDL2 (a entrada com o processo do jogo em segundo plano está ligada para o launcher; ainda não testada assim com um controle físico); 🧪 vibração (`PS3_RUMBLE`, `PS3_RUMBLE_STRENGTH`) portada, precisa de um teste jogando |
| Teclado e mouse | 🧪 portados do trabalho do inFAMOUS 2: um layout de FPS (WASD, mouse para olhar, clique esquerdo atira), remapeável no `input.toml` (`PS3_INPUT_MAP`, a página Controles do launcher); testes passam, precisa de um teste jogando |
| Modos de janela, vsync, MetalFX, SMAA e filtros, pacotes de texturas, resolução interna | 🧪 portados do trabalho do inFAMOUS 2 (veja [Variáveis de ambiente](#variáveis-de-ambiente)); testes passam, precisa de um teste jogando; a resolução interna é experimental |
| Atualização v1.7 do jogo | ✅ a campanha roda o `game.self` da atualização, traduzido de novo a partir do `patch_06`: menus → Nova campanha → a abertura de York, gameplay e renascimento verificados; os menus continuam o EBOOT 01.00 do disco (veja [A campanha v1.7](#a-campanha-v17)); vibração com um controle físico, uma sessão longa e uma medição de fps da v1.7 ainda não verificadas |
| App launcher | ✅ [`launcher/`](launcher/) (Tauri): disco, builds, recompilação, ajustes, saves, Jogar/Parar, em português do Brasil e inglês; 🧪 a nova página Controles (remapeamento, mouse, vibração) e os ajustes de Gráficos (janela, filtros, pacotes de texturas) |
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
  `SPU_RunCntl`=0 na SPU raw 1 (o segundo programa de SPU raw), traz o programa
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
- **Desempenho** (2026-10-23): o jogo amostra o seu buffer de profundidade
  de 2560×720 como uma textura A8R8G8B8 a cada quadro. Isso era uma leitura
  GPU→CPU e um reenvio de 7,4 MB por quadro; agora é resolvido na GPU
  (`PS3RECOMP_ENG_ZRGBA_GPU=0` desliga): a thread do RSX foi de 54% para 12%
  ocupada e os reenvios de textura a cada 120 quadros de ~100 para 0. A
  checagem de store-watch da PPU tem um caminho rápido (era 28% da thread
  principal) e os relógios do guest leem o `mach_absolute_time`. No código
  SPU: aritmética de precisão simples nativa com arredondamento para zero
  (exata bit a bit; `SPU_FP_NATIVE=0` desliga), um `goto` local para os
  desvios calculados dentro de uma função (`SPU_LOCAL_IB=0`), chamadas
  diretas a alvos traduzidos conhecidos (`SPU_DIRECT`) e retornos de chamada
  mais baratos, medidos em cerca de +5% (32,2/33,0 fps ligado contra 30,7
  desligado, com ruído). A campanha agora chega ao limite do próprio jogo,
  ~30–33 fps; o tempo "ocupado" que resta na PPU é em boa parte o limitador
  de quadros do jogo. Os jobs SPU mais pesados rodam numa SPU só porque o
  jogo os prende nela (workload 5, contention 1..1), como no hardware.
- **A campanha v1.7:** o `patch_06` da atualização traz um `game.self` novo.
  Ele é traduzido como o do disco e compilado como `build/BCUS98107_game_v17`;
  no exitspawn o `port/stubs.cpp` o executa no lugar do build 01.00 quando ele
  e o ELF dele existem. 13 dos 15 módulos de job WWS e os dois programas das
  SPUs raw mudaram e são traduzidos a partir do ELF v1.7; o policy module é
  o mesmo.
- **Continuar a partir de um checkpoint** (travava ao carregar): o segundo
  programa de SPU raw tem uma área de parâmetros de 0x50 bytes que o PPU
  preenche antes de cada carga (ponteiros do heap, valores da fase, a janela
  MMIO da própria SPU raw), então os bytes com que ele é carregado, e a
  fingerprint deles, mudam com a fase, o checkpoint e o número da SPU; um
  Continuar carregava uma que nenhum build tinha registrado. O runtime agora
  reconhece o programa com essa área mascarada
  (`spu_workload_register_img_masked`, patch do toolkit), e os dois programas
  são recortados do ELF pelo cabeçalho `0xDEADDEAD` que o carregador usa
  (`tools/extract_rawspu_elf.py`), então não é preciso um run de captura.
- **Som dos vídeos:** o cellAdec era um stub que não decodificava nada, então
  todo vídeo era mudo. Agora ele decodifica o AC3 5.1 dos vídeos pelo mesmo
  FFmpeg carregado em tempo de execução que o vídeo usa
  (`libs/codec/adec_ffmpeg.c`, com a fila, as informações do bitstream e os
  timestamps), e o mixer POSIX se cadencia pelo relógio (um bloco a cada
  5,333 ms, ajustes de no máximo um quarto de período, 4 ms de tolerância
  para um produtor atrasado, 7.1 reduzido para estéreo sem clipping, o nível
  inicial de cada porta respeitado). O vídeo de abertura agora toca a
  −25…−34 dBFS nos seis canais.
- **Névoa, fumaça e a água falsa:** o `WPOS` do RSX segue o
  `SET_SHADER_WINDOW`, cuja origem aqui é embaixo à esquerda; o
  descompilador de fragment programs o ignorava, então a névoa no chão, a
  fumaça e mais um shader, que consultam o buffer de profundidade em
  720 − `WPOS.y`, o liam de cabeça para baixo, e a fase do rio mostrava um
  "reflexo" com listras verticais sobre o leito seco de um riacho (e sobre um
  corpo e uma caixa de munição). Agora a janela é aplicada ao `WPOS` nos
  programas que o leem e faz parte da chave do pipeline deles. Os shadow maps
  de profundidade de 16 bits em ordem swizzled (Morton) são gravados de volta
  na memória do guest, então os soldados distantes não ficam mais sobre
  retângulos escuros.

As mudanças no toolkit são um único patch sobre o ps3recomp v0.17.0,
[patches/ps3recomp-a6d76c8-resistance.patch](patches/ps3recomp-a6d76c8-resistance.patch):
a informação de pilha da thread principal, o contexto de kernel SPURS que o
policy module lê, o offset de vértice de 28 bits, o cache de vértices e o
resto do trabalho de desempenho, as correções de uma comparação com o RPCS3
(direção da comparação das sombras, snapshots de profundidade recortados e
liberados, profundidade replicada em todas as lanes, texturas comprimidas
nunca resolvidas a partir de render targets, threads PPU começando com
VSCR[NJ]=1, sem `getenv` a cada escrita em canal MFC da SPU), a parada e o
reinício das SPUs raw pelo `SPU_RunCntl` e a fingerprint mascarada das SPUs
raw, a decodificação AC3 do cellAdec e a cadência do mixer, o `WPOS` com a
janela do shader e a gravação de volta da profundidade swizzled, a janela por
IOSurface compartilhada
(`libs/video/rsx_host_surface.h`, `tools/host_window/host_window.m`; a
janela recolhe o jogo com um `waitpid` bloqueante, então sai quando o jogo
sai, que é do que o Parar do launcher depende; o "guest FPS" do título conta
cada flip), a passagem do estado da janela pelo exitspawn, a resolução da
profundidade como RGBA8 na GPU e as acelerações de PPU/SPU acima, o que o
trabalho do inFAMOUS 2 acrescentou depois (teclado e mouse com remapeamento
pelo `input.toml`, vibração, modos de janela, vsync e MetalFX na janela, SMAA
com os shaders de licença MIT em `third_party/smaa`, os filtros e o LUT da
passada de apresentação, pacotes de texturas, a resolução interna) e
diagnósticos (`VDEC_PICLOG`,
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
| `lift` | o `ppu_lifter.py` do toolkit, 16 pedaços para o EBOOT e 12 para o `game.self`, com as instruções comutáveis dos patches de 60 fps (`port/patches/*.insn.json`) | `recompiled/BCUS98107/`, `recompiled/BCUS98107/game_module/` |
| `spu` | `tools/regen_spu_pm.sh`: o policy module do job manager, extraído do seu EBOOT, embrulhado e traduzido; `tools/regen_jobmods.sh`: os 15 módulos de job WWS; `tools/regen_rawspu.sh`: os dois programas das SPUs raw, recortados do `game.elf` | `port/spu/captured/`, `port/spu/jobmod_*/`, `port/spu/rawspu0/`, `port/spu/rawspu1/` |
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

Um lift feito antes de 2026-10-23 não tem o patch de 60 fps (`RES_UNLOCK_FPS`
e `RES_MENU_60FPS` então não fazem nada): depois de atualizar o toolkit, rode
`tools/relift.sh` (ele traduz de novo o EBOOT e as campanhas 01.00 e v1.7 com
`--insn-patches` e recompila; só um ou dois pedaços mudam).

As correções do Continuar, do som dos vídeos e da névoa estão no patch do
toolkit e no registro das SPUs: depois do `git pull`, apague `ps3recomp/` e
rode `tools/rebuild.sh toolkit spu build`, mais
`tools/v17/regen_v17.sh spu build` para a campanha v1.7 (ele copia o registro
mascarado para `port/spu_v17/`). O run de captura das SPUs raw
(`SPU_RAW_DUMP_DIR`) não é mais necessário.

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

Quando uma fase começa, o `game.self` carrega dois programas em SPUs raw por
DMA proxy (local store a partir de 0x60, entrada em 0x90). Os dois estão
linkados no segmento de dados dele, e o carregador do PPU só recebe esses dois
(pares {ponteiro, tamanho} constantes no GOT), então nenhuma fase carrega
outro. O `tools/extract_rawspu_elf.py` os recorta do ELF e eles são traduzidos
na base 0x60 (a etapa `spu` do `tools/rebuild.sh` faz isso):

```sh
tools/regen_rawspu.sh game/BCUS98107/modules/game.elf   # -> port/spu/rawspu0, port/spu/rawspu1
```

O `port/spu/spu_workloads.c` registra os dois como imagens SPU 2
(`1894F66385AFD052`) e 3 (`2B2DB6A5DE233A5F`). O segundo tem uma área de
parâmetros de 0x50 bytes que o PPU preenche antes de cada carga (ponteiros do
heap, valores da fase, a janela MMIO da própria SPU raw), então os bytes com
que ele é carregado, e a fingerprint deles, mudam com a fase, o checkpoint e o
número da SPU. Ele é registrado com essa área mascarada
(`spu_workload_register_img_masked`): a fingerprint é a do programa com a área
zerada, como está no ELF. Como o policy module, o C traduzido é código do jogo
e fica fora do git.

#### A campanha v1.7

A última atualização do jogo, a v1.7, são sete arquivos,
`patch_01`…`patch_07.psarc`; o `patch_06` traz um executável de campanha
novo (`/data/new_game.self`). O port o roda. O ELF descriptografado dele
(SHA-256 `e1d75e6d4afe…144f`, o "game.self Updated" do RPCS3,
PPU-a08a187d…) é traduzido como o do disco (19.706 funções contra 19.219), e
13 dos 15 módulos de job WWS e os dois programas das SPUs raw mudaram e são
traduzidos a partir dele (fingerprints novas `6D03497950639952` e
`F9726FDA4A599084`, esta com a área de parâmetros mascarada como na 01.00; o
policy module é idêntico byte a byte). Ele importa
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
```

O `regen_v17.sh` extrai o `new_game.self` do `patch_06`
(`tools/v17/psarc_extract.py`), descriptografa com `rpcs3 --decrypt` para
`game/BCUS98107/modules_v17/game.elf` (conferindo o SHA-256), traduz para
`recompiled/BCUS98107/game_module_v17/`, traduz o código de SPU dele para
`port/spu_v17/` (o policy module é copiado de `port/spu/captured/`, então
rode antes a etapa `spu`; os programas das SPUs raw são recortados do ELF
v1.7, como na 01.00) e compila `build/BCUS98107_game_v17/`.

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
DualShock ou Xbox) ou use teclado e mouse, e:

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

Teclado e mouse (novos, ainda sem teste jogando; um clique na janela captura
o mouse, Esc o solta):

| Ação | Teclas e mouse | PS3 |
| --- | --- | --- |
| Andar / olhar | W A S D / o mouse (ou I J K L) | analógico esquerdo / direito |
| Atirar / mirar | clique esquerdo ou X / clique direito ou Z | R1 / L1 |
| Granada | G ou botão 4 do mouse | L2 |
| R2 | clique do meio ou 1 | R2 |
| Pular / agachar | Espaço / C, Ctrl ou Backspace | ✕ / ○ |
| Recarregar, usar | R, E ou botão 5 do mouse | □ |
| Trocar arma | Q ou a roda | △ |
| L3 / R3 | Shift / V | L3 / R3 |
| Start / select | Return, Esc ou P / Tab ou M | START / SELECT |

Troque as teclas na página Controles do launcher, ou num `input.toml`
passado como `PS3_INPUT_MAP` (o formato está no `libs/input/pad_input_map.h`
do toolkit).

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
- **Gráficos**: modo de exibição (janela, tela cheia do macOS, sem bordas),
  tamanho da janela (o `--size` da janela), tela, vsync, o ícone do seu disco
  no Dock; resolução interna (experimental), ampliação MetalFX, FXAA ou SMAA,
  nitidez, filtragem anisotrópica, gama forçado, perfis; filtros (vibrance,
  contraste, vinheta, granulado, um arquivo de LUT de cor); pacotes de
  texturas (ligar e ordenar os pacotes da pasta de mods, extrair as texturas
  do jogo).
- **Desempenho**: o cache de vértices e a divisão por fluxo, ligar só o
  estado que mudou, o cache de shaders (com o tamanho e Limpar) e os shaders
  em segundo plano, o relógio de quadros, o contador de fps, manter o Mac
  acordado; **60 fps** (`RES_UNLOCK_FPS` + `RES_MENU_60FPS`) e o campo de
  visão (`RES_FOV`), que aparecem quando o build os tiver.
- **Jogo**: o disco (imagem ou pasta, montar/ejetar), a versão da campanha
  (**v1.7** quando o build dela existe, ou **01.00**: `RES_GAME_VERSION=100`),
  o idioma do console (`PS3_LANG`), os vídeos (caminhos do FFmpeg), o áudio
  (volume, sem som, saída), a pasta de jogo e um gerenciador de saves (o perfil e o `quick.sav`, aqui e no RPCS3:
  backup antes de cada Jogar, restaurar, copiar nos dois sentidos,
  exportar/importar .zip).
- **Controles**: as associações de teclado, mouse e controle com o que cada
  botão faz no jogo (gravadas em `input.toml`), um teste do controle, o que
  controla o jogo (os dois, só o controle, só teclado e mouse), sensibilidade,
  inversão, zona morta e captura do mouse, vibração e a intensidade dela.
  Cada ajuste só chega ao jogo quando é diferente do padrão.
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
| `PS3_VBLANK_HZ=<10..1000>` | a taxa do vblank sintetizado (padrão 62,5); com `RES_UNLOCK_FPS=1`, 120 deixa a campanha rodar a ~100 fps |
| `RES_UNLOCK_FPS=1`, `RES_MENU_60FPS=1`, `RES_FOV=<0.1..4>` | os patches opcionais do jogo ([`port/patches.cpp`](port/patches.cpp)): o limitador de quadros da campanha desligado (passo mínimo 1/30 → 1/240 s), os menus e o multiplayer a um vblank por quadro, o fator de campo de visão (1,0 = o do jogo; menor é mais aberto). Cada um só se aplica quando o ELF em execução tem as palavras que ele espera |
| `PS3_AUDIO_VOLUME=0..100`, `PS3_AUDIO_DEVICE=<nome>`, `PS3_AUDIO_MUTE=1` | volume geral (100), a saída cujo nome contém `<nome>` (as saídas são listadas no log quando nenhuma corresponde; macOS/Linux), sem som (qualquer valor silencia) |
| `PS3_AUDIO_DUMP=<arquivo.wav>`, `PS3_AUDIO_STATS=1` | grava a mistura final (48 kHz estéreo float; antes do mudo e do volume; os menus e a campanha vão para um arquivo só) / a cada ~5 s, registra no log o nível de cada porta de áudio e da mistura, os blocos em silêncio e as amostras cortadas |
| `PS3_LICENSE_AREA=<n>` | a área de licença que o console informa (`CELL_SYSUTIL_LICENSE_AREA_*`; padrão América do Norte, a região do disco) |
| `PS3_USLEEP_SPIN_US=<µs>` | um `sys_timer_usleep` de até tantos µs gira (cedendo a vez) em vez de dormir (opcional; threads girando gastam núcleos de que as SPUs podem precisar) |
| `PS3_INPUT_MAP=<arquivo>` | associações de teclado, botões do mouse e controle e os ajustes `[mouse]` (`input.toml`; formato no `libs/input/pad_input_map.h` do toolkit) |
| `PS3_INPUT_MODE=both\|controller\|keyboard` | o que controla o jogo (padrão both); `PS3_PAD_SDL=0` ignora controles físicos |
| `PS3_MOUSE=0`, `PS3_MOUSE_SENS=<f>`, `PS3_MOUSE_INVERT_X=1`, `PS3_MOUSE_INVERT_Y=1`, `PS3_MOUSE_DEADZONE=0..120`, `PS3_MOUSE_CAPTURE=0` | a câmera no mouse: desligada, velocidade (1,0), inversão, primeiro passo do analógico além do centro (30), sem capturar o cursor |
| `PS3_RUMBLE=0`, `PS3_RUMBLE_STRENGTH=0..200` | vibração desligada / a intensidade em porcento (100) |
| `PS3_WINDOW_MODE=windowed\|fullscreen\|borderless`, `PS3_WINDOW_SIZE=LxA`, `PS3_DISPLAY=<n\|nome>`, `PS3_VSYNC=0`, `PS3_UPSCALE=metalfx` | a janela: como ela abre (com faixas pretas; o `--size` continua valendo), em qual tela, vsync desligado, ampliação MetalFX |
| `PS3_AA=fxaa\|smaa`, `PS3_SHARPEN`, `PS3_ANISO`, `PS3_DISPLAY_GAMMA` | antisserrilhamento, nitidez (0–1), filtragem anisotrópica, gama da tela |
| `PS3_VIBRANCE=-1..1`, `PS3_CONTRAST=0.5..1.5`, `PS3_VIGNETTE=0..1`, `PS3_GRAIN=0..1`, `PS3_LUT=<png>`, `PS3_LUT_STRENGTH=0..1` | os filtros da passada de apresentação e um LUT de cor (o formato PNG do ReShade) |
| `PS3_MODS=<pacote,...>`, `PS3_MODS_DIR=<pasta>`, `PS3_MODS_DUMP=<pasta>` | pacotes de texturas (pastas de `<chave>.png`, o primeiro vence) e a extração de cada textura com a sua chave |
| `PS3_RES_SCALE=1\|1.5\|2\|3\|4` (ou `720p`, `1080p`, `1440p`, `4k`) | resolução interna (experimental) |
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
| [`port/`](port/) | O port: `main.cpp` (boot, thread do guest, bloco de dados do exitspawn), `stubs.cpp` (exitspawn e a passagem do estado da janela, `RES_PEEK`), `patches.cpp` e `patches/*.insn.json` (os patches opcionais de 60 fps / FOV), `CMakeLists.txt` (com o alvo do launcher `Resistance`), `config.toml`, `spu/spu_workloads.c` (registro das SPUs); `spu_v17/` é gerado pelo `tools/v17/regen_v17.sh` |
| [`patches/`](patches/) | O patch do toolkit sobre o ps3recomp v0.17.0 |
| [`tools/`](tools/) | `setup_toolkit.sh`, `rebuild.sh`, `relift.sh`, `regen_spu_pm.sh`, `extract_pm.py`, `regen_rawspu.sh`, `extract_rawspu_elf.py`, `play.sh` e os scripts de execução; `v17/` (a campanha v1.7: `regen_v17.sh`, `psarc_extract.py`, `spu_workloads.c`) |
| [`launcher/`](launcher/) | O launcher de desktop (Tauri): ajustes, disco, saves, recompilação e Jogar |
| [`ROADMAP.pt-BR.md`](ROADMAP.pt-BR.md) | Estado e próximos passos |

## Licença

Código deste repositório: MIT ([LICENSE](LICENSE)). O toolkit é MIT pelos
seus autores. Resistance: Fall of Man © Sony Interactive Entertainment LLC,
desenvolvido pela Insomniac Games; este projeto não é afiliado à Sony nem à
Insomniac.
