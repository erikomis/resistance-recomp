# Roadmap

🌐 [English](ROADMAP.md) · **Português (BR)**

Estado do port nativo de Resistance: Fall of Man (`BCUS98107`, disco v01.00,
campanha da atualização v1.7) para macOS/arm64 e o que vem a seguir. Mesmo
formato do roadmap do inFAMOUS 2.
`[x]` = feito e verificado; `[ ]` = a fazer; "(em andamento)" = sendo feito.

## Resumo (2026-10-24)

- **Roda até o gameplay:** os dois executáveis estão traduzidos, compilam e rodam no macOS/arm64 com Metal. Boot, telas legais, instalação da primeira execução, tela de título e menu principal levam a uma campanha nova. O `game.self` então toca o vídeo de abertura da fase 1 e carrega o capítulo 1, "The Gauntlet" (York).
- **Campanha v1.7:** a campanha é a da última atualização do jogo. O `game.self` dela (do `patch_06`) foi traduzido de novo e roda a partir dos menus da 01.00: Nova campanha → abertura de York, gameplay e renascimento verificados. Os menus continuam os da 01.00.
- **Jogado:** o capítulo 1 numa janela com um DualSense (SDL2), `tools/play.sh`: primeira pessoa, HUD, inimigos e aliados, tiros, morrer e renascer no chão, no checkpoint (o bug do renascimento foi corrigido). A cidade aparece com as cores certas (o mundo azul foi corrigido).
- **Uma janela:** o launcher `Resistance` é dono da janela e o jogo desenha nela por IOSurfaces compartilhadas; a mesma janela fica aberta dos menus até a abertura da campanha (~33 fps).
- **App launcher:** o `launcher/` (Tauri) acha os builds e o disco, recompila, guarda os ajustes e os saves, e abre e fecha o jogo.
- **Sombras e fumaça:** os retângulos escuros no chão e a fumaça de bordas duras sumiram no jogo.
- **No limite do próprio jogo:** a campanha chega ao limite de ~30–33 fps do jogo (o Resistance roda a 30 fps no PS3); o tempo "ocupado" que resta na PPU é em boa parte o limitador de quadros do jogo. A resolução da profundidade como RGBA8 na GPU levou a thread do RSX de 54% para 12% ocupada (2026-10-23).
- **Novo, precisa de um teste jogando:** teclado e mouse no macOS com remapeamento pelo `input.toml`, vibração, modos de janela/vsync/MetalFX, SMAA e filtros, pacotes de texturas e a resolução interna (experimental), portados do trabalho do inFAMOUS 2; os testes passam. O launcher tem páginas Controles e Gráficos para eles.
- **60 fps e campo de visão (2026-10-23):** os patches da comunidade para o RPCS3 trazidos para o port (`port/patches.cpp`, `port/patches/*.insn.json`), todos opcionais: `RES_UNLOCK_FPS=1` na campanha (01.00 e v1.7) na velocidade normal do jogo, `RES_MENU_60FPS=1` menus de 32 → 59 fps, `RES_FOV`; o botão "60 fps" do launcher liga os dois.
- **Cadência a 60 fps (2026-10-24):** com `RES_UNLOCK_FPS=1` o port agora roda o relógio de vblank a 120 Hz (`PS3_VBLANK_HZ`, ainda ajustável): com ticks de 16 ms um quadro de 17 ms esperava o tick seguinte ao próximo. Medido no gameplay: 88,5 fps (eram ~50–56), velocidade do jogo inalterada. O `PS3_FLIP_MAX_HZ` (desligado por padrão) limita os flips; limitar a 60 mediu pior (50 fps).
- **Sombras no chão (2026-10-24):** retângulos escuros de bordas duras no chão: as tabelas de superfícies e de profundidade do draw engine (64 vagas cada) enchiam e nada era descartado. O Resistance empacota os shadow maps num pool por quadro cujos endereços mudam, então, com as tabelas cheias, as passadas de sombra novas eram descartadas em silêncio e quem recebe a sombra lia um mapa vazio. As tabelas agora descartam a entrada usada há mais tempo; os descartes aparecem no log de uma sessão de jogo, falta um teste jogando na cena.
- **Tela dividida cooperativa (2026-10-24):** a visão do jogador 1 (esquerda) ficava preta: a limpeza de superfície ignorava o scissor, então a limpeza do jogador 2 apagava a metade do jogador 1. As limpezas agora respeitam o scissor (passadas de limpeza parcial no Metal); as duas metades aparecem no jogo.
- **Polygon offset e posições invariantes (2026-10-24):** o polygon offset é respeitado (era ignorado; o jogo usa nos shadow maps dos personagens, nos decalques do terreno e nas sombras projetadas), e a posição do vértice é compilada como invariante (a pré-passada de profundidade da vegetação e a passada de cor com `EQUAL`). As contagens reais de oclusão ZPASS estão implementadas, mas desligadas por padrão (`PS3RECOMP_ENG_ZPASS=1`): custam taxa de quadros a 60 fps e não mostraram ganho visível.
- **Salvar e Sair a 60 fps (depois da v0.2.1, na main):** o desbloqueio punha o vblank de 120 Hz no ambiente, e o exitspawn é um `execv`, então o Salvar e Sair o levava de volta aos menus, cujos passos fixos de 1/60 podiam então rodar em velocidade dobrada. A taxa agora é passada dentro do processo (`g_res_vblank_hz`); o `PS3_VBLANK_HZ` ainda tem prioridade.
- **Robustez (2026-10-24):** mais de duas horas de jogo roteirizado sem nenhum crash: todos os menus e extras do EBOOT, multiplayer em tela dividida offline, pausa e opções, Salvar e Sair → Continuar (exitspawn nos dois sentidos), morrer e renascer a 30 e 60 fps, um HDD novo e vazio, um `quick.sav` corrompido, 25 minutos seguidos a 60 fps e 22 minutos parado. Multiplayer → Online Game não trava mais offline (o diálogo de início de rede responde "cancelado"; tickets do NP adicionados), o NID chamado ao sair é o `cellSysconfAbort` (adicionado junto com o `cellSysconfOpen`), e `cellPadInfoSensorMode`/`cellPadSetSensorMode` entraram: um run completo não tem NIDs sem resolver.
- **Telas legais corrigidas (2026-10-23):** o boot vai dos logos ao título "Press the START button" sem texto sobreposto; o runtime dizia ao jogo que ele estava num console japonês (área de licença).
- **Áudio (2026-10-23):** o mixer do macOS segue o tempo real (alternava 2,4× / 0,95×); volume, sem som e saída de áudio (`PS3_AUDIO_*`, também no launcher).
- **Continuar corrigido (2026-10-23):** Continuar a partir de um checkpoint travava ao carregar (v1.7 e 01.00): o segundo programa de SPU raw tem uma área de parâmetros que muda com a fase e o checkpoint, então ele chegava com uma fingerprint que nenhum build tinha registrado. Agora ele é reconhecido com essa área mascarada, e os dois programas das SPUs raw são recortados do ELF (sem run de captura). Continuar a partir de um checkpoint salvo, campanha nova e renascimento verificados nas duas campanhas.
- **Som dos vídeos (2026-10-23):** todo vídeo era mudo (o cellAdec não decodificava nada); as faixas AC3 5.1 agora são decodificadas pelo FFmpeg e o mixer POSIX se cadencia pelo relógio. O vídeo de abertura toca a −25…−34 dBFS nos seis canais, 0–0,2% de blocos em silêncio.
- **Névoa, fumaça e a água falsa (2026-10-23):** a fase do rio mostrava um "reflexo" com listras verticais: o `WPOS` dos fragment shaders ignorava a origem embaixo à esquerda da janela, então os shaders de névoa no chão e de fumaça liam o buffer de profundidade de cabeça para baixo. A fase agora mostra o leito seco do riacho com névoa suave (nas duas campanhas). Os soldados distantes não ficam mais sobre retângulos escuros (shadow maps de profundidade de 16 bits swizzled gravados de volta na memória do guest).
- **Toolkit:** 18 correções de pull requests do ps3recomp upstream e do trabalho no inFAMOUS 2 (veja CREDITS); um run de regressão com todas elas renderiza sem erros, renascer OK.
- **Lançado:** v0.1.0 e v0.2.0 (2026-10-23) e v0.2.1 (2026-10-24), pré-releases: o launcher para macOS e uma prévia para Windows.
- **Retribuindo (2026-10-24):** mais cinco pull requests no upstream: limpezas com scissor (#560), o polygon offset (#562), posições invariantes (#563), `cellSysconfOpen`/`Abort` (#564), o modo de sensor do controle (#565) (veja [Retribuindo](#retribuindo)).
- **Em andamento:** uma varredura de todas as fases da campanha: carregar cada uma, conferir renderização, jobs de SPU, chamadas HLE e fps, corrigir o que quebrar.
- **Em aberto:** pontos escuros na vegetação (parece ser o conteúdo do próprio jogo, o alpha granulado das folhas; ainda não comparado com o RPCS3); um defeito nos olhos de um personagem que não foi reproduzido (precisa de um jeito de reproduzir); a música do menu às vezes tem conteúdo a mais no centro/LFE (+4–6 dB, do sintetizador do jogo, varia a cada execução); os menus continuam os da 01.00 (o EBOOT da v1.7 só existe em NPDRM); teclado, mouse, vibração e os novos ajustes de gráficos ainda não foram usados jogando; um controle físico com o launcher; Windows não testado (o jogo, e o novo mixer de áudio), sem Linux.
- **Próximo:** terminar a varredura das fases; um jeito de carregar qualquer fase direto (uma ferramenta); um teste jogando da correção do pool de shadow maps na cena onde os retângulos apareciam; reproduzir o defeito nos olhos; comparar a vegetação com o RPCS3; uma sessão de jogo com teclado e mouse, vibração e os novos ajustes de gráficos; um controle físico com o launcher; Windows.

## Estado atual (2026-10-24)

O pipeline inteiro roda a partir do disco do próprio jogador
(`tools/rebuild.sh`): a imagem do disco é montada e usada como raiz do VFS
(`PS3_VFS_ROOT`), os dois executáveis PPU são descriptografados, analisados,
traduzidos (EBOOT com 25.022 funções; `game.self` com 18.994) e compilados
contra o ps3recomp v0.17.0 com o patch do toolkit deste repositório.

O que trouxe até aqui:

- **Job manager:** o policy module do job manager SPURS do jogo (fingerprint
  `4E4076E700C6AFF3`, 11.136 bytes) é traduzido na base real de LS, 0xA00
  (`tools/regen_spu_pm.sh`). Antes ele ficava preso em `GETLLAR 0x2480`.
- **Metal fora da thread do jogo:** o jogo roda numa thread própria e a thread
  principal do macOS gira o run loop (`port/main.cpp`). Com o jogo na thread
  principal, o pump de eventos do Metal entrava em deadlock e o FIFO nunca
  era drenado.
- **exitspawn:** `sys_game_process_exitspawn` (`port/stubs.cpp`) faz execv do
  outro build. O bloco de dados do pai vai para topo-da-pilha-principal − 0x1000,
  onde o `game.self` procura o bloco `"ELFtastic!"`, e
  `sys_ppu_thread_get_stack_information` agora informa a pilha principal real
  em vez de 0 + 0 (patch do toolkit).
- **Vídeos:** o cellVdec decodifica pelo FFmpeg carregado em tempo de execução
  (`-DPS3RECOMP_FFMPEG_RUNTIME=ON`, FFmpeg do Homebrew). O quad do vídeo é
  desenhado em modo imediato com posições `DATA2S`, que o dispatch do RSX não
  transformava em vértices, por isso a tela ficava preta (patch do toolkit).
  O libavcodec segura a última imagem reordenada por B-frames. O player espera
  por ela antes de encerrar o vídeo, então o cellVdec esvazia o decodificador
  quando o jogo já pegou todas as imagens e não envia AU há 1,5 s (patch do
  toolkit).
- **SPUs raw:** depois do vídeo, o `game.self` carrega dois programas em SPUs
  raw. As SPUs raw só eram iniciadas no Windows. A arena do guest também
  parava abaixo das janelas delas em `0xE0000000`, e toda SPU raw depois da
  primeira rodava o programa da primeira. As três coisas foram corrigidas
  (patch do toolkit, `port/main.cpp`). Os dois programas estão linkados no
  ELF; o `tools/extract_rawspu_elf.py` os recorta pelo cabeçalho `0xDEADDEAD`
  que o carregador usa e o `tools/regen_rawspu.sh` os traduz (antes eram
  capturados num run com `SPU_RAW_DUMP_DIR`).
- **O mundo azul:** o RSX soma o `VERTEX_DATA_BASE_OFFSET` ao offset de cada
  array de vértices e faz o wrap da soma em 28 bits (o
  `get_vertex_offset_from_base` do RPCS3; as engines da Insomniac dependem
  disso). O stream de iluminação do mundo do Resistance, refeito a cada frame
  (atributo 4, UB256, memória local), fica em base + offset = 0x1Fxxxxxx,
  local 0x0Fxxxxxx. Sem o wrap o port lia além da VRAM, pegava zeros, os pesos
  de iluminação saíam (0,0,1) e o mundo ficava azul. O
  `rsx_dsp_resolve_vertex_offset` (patch do toolkit) é usado pelos dois
  caminhos de fetch de vértices, com um teste de regressão em
  `test_vertex_fetch_plan.c` que falha sem ele.
- **Velocidade:** o cache de vértices entre frames do port do inFAMOUS 2 (5,9
  → 14 fps no gameplay, mesmo build, cache ligado vs desligado). O port vinha
  sendo compilado em -O0 (sem `CMAKE_BUILD_TYPE`, e o -O2 do toolkit só valia
  para Release/RelWithDebInfo); agora o runtime é -O2 e o código PPU traduzido
  -O1 em toda configuração menos Debug. Com o pacote de desempenho do
  inFAMOUS 2: 32,5 fps nos primeiros 60 s de gameplay contra 6,0 fps com as
  chaves dele desligadas (`PS3RECOMP_ENG_VCACHE_SPLIT=0`,
  `PS3RECOMP_METAL_BIND_ALL=1`, `PS3_FRAMECLOCK_US=4000`), e 30,5 fps na
  cidade com todas as correções.
- **Renascimento:** a cada recarga do checkpoint o jogo escreve
  `SPU_RunCntl`=0 na SPU raw 1 (o segundo programa de SPU raw), traz o programa de
  volta para o local store por DMA proxy e escreve `SPU_RunCntl`=1. O runtime
  ignorava a parada enquanto a SPU estava parada esperando num canal e
  descartava o reinício sem avisar, então o programa antigo continuava rodando
  sobre a imagem recarregada e o jogador ia direto para o céu. Como no
  `RawSPUThread.cpp` do RPCS3, `SPU_RunCntl`=0 agora para de fato a SPU raw (o
  pedido de parada é respeitado nas leituras e esperas de canal e na espera da
  mailbox de saída; o `raw_spu_stop` espera a thread do host) e
  `SPU_RunCntl`=1 a começa do zero (`runtime/spu/spu_raw.c`, patch do
  toolkit).
- **Uma janela:** um launcher pequeno, `Resistance`
  (`tools/host_window/host_window.m`), é dono da janela. O `ResistanceRecomp`
  renderiza na GPU em IOSurfaces compartilhadas com ele (o nome de bootstrap
  Mach vai em `PS3RECOMP_HOST_SURFACES`) e não tem janela nem ícone no Dock. A
  janela continua aberta no exitspawn do EBOOT para o `game.self` (o `execv`
  mantém o pid e a imagem nova se conecta de novo). ~0,35 ms de GPU por frame,
  sem cópia na CPU, sem latência a mais mensurável. A janela recolhe o jogo
  com um `waitpid` bloqueante: um recolhimento sem bloqueio podia rodar um
  instante antes de o jogo poder ser recolhido, e a janela continuava aberta
  depois de o jogo ter saído (o Parar do launcher esbarrava nisso). Agora um
  SIGTERM ao grupo de processos fecha os dois em ~3 s. O "guest FPS" do
  título conta cada flip (mostrava 0).
- **A campanha v1.7:** a última atualização do jogo são sete arquivos,
  `patch_01`…`patch_07.psarc`; o `patch_06` traz um `game.self` novo
  (`/data/new_game.self`; o SHA-256 do ELF descriptografado é
  `e1d75e6d4afe…144f`, o "game.self Updated" do RPCS3, PPU-a08a187d…). Ele foi
  traduzido de novo (19.706 funções contra 19.219 da 01.00) e é compilado como
  `build/BCUS98107_game_v17`; 13 dos 15 módulos de job WWS e os dois
  programas das SPUs raw mudaram e foram traduzidos de novo (fingerprints
  novas `6D03497950639952` e `F9726FDA4A599084`, esta com a área de
  parâmetros mascarada; o policy module é idêntico byte a byte). Imports novos: `cellSaveDataAutoSave2`/`AutoLoad2`,
  `_sys_process_atexitspawn`, `cellPadSetActDirect` (vibração). No exitspawn
  o `port/stubs.cpp` roda o build v1.7 quando ele e o ELF dele existem
  (`RES_GAME_VERSION=100` mantém a 01.00). O `tools/v17/regen_v17.sh`
  (`tools/rebuild.sh v17`) extrai o `patch_06`, descriptografa o
  `new_game.self` com `rpcs3 --decrypt`, traduz, traduz os programas de SPU
  (os das SPUs raw recortados do ELF) e compila. Verificado: menus
  (EBOOT 01.00) → Nova campanha → o log mostra `campaign = v1.7 game.self` →
  a abertura de York, "The Gauntlet"; gameplay e renascimento funcionam com a
  correção do `SPU_RunCntl`.
- **Os menus continuam os da 01.00:** o `EBOOT.BIN` da atualização só existe
  na forma NPDRM (edição digital). O EBOOT 01.00 do disco não roda com os
  dados da atualização v1.7 montados (`patch_04`/`patch_06`: cai ao carregar
  as fontes do menu; 1/2/3/5/7: trava no exitspawn), então os arquivos da
  atualização não podem estar na pasta de jogo; o `tools/play.sh` e o
  launcher se recusam a começar enquanto houver um `patch_NN.bin` lá. A
  campanha não precisa deles: são dados do front end e do multiplayer, e o
  `game.self` nunca monta os arquivos da atualização. Os arquivos da
  atualização são as atualizações oficiais e gratuitas do jogo; cada jogador
  guarda os seus (o PSRewired, psrewired.com, deixa quem tem o jogo baixar a
  atualização pelo próprio jogo no RPCS3).
- **Sombras e fumaça:** os retângulos escuros no chão e a fumaça de bordas
  duras sumiram no jogo (conferido visualmente) depois das correções de
  profundidade da comparação com o RPCS3: a comparação das sombras é
  profundidade gravada OP referência, as texturas de profundidade replicam a
  profundidade em todas as lanes e os snapshots de profundidade são
  recortados à região que a textura indica.
- **Desempenho (2026-10-23):** o jogo amostra o seu buffer de profundidade
  de 2560×720 como uma textura A8R8G8B8 a cada quadro; isso era uma leitura
  GPU→CPU e um reenvio de 7,4 MB por quadro e agora é resolvido na GPU
  (`PS3RECOMP_ENG_ZRGBA_GPU=0` desliga): thread do RSX de 54% → 12%
  ocupada, reenvios de textura a cada 120 quadros de ~100 → 0. Um caminho
  rápido para a checagem de store-watch da PPU (era 28% da thread principal)
  e relógios do guest pelo `mach_absolute_time`. SPU: precisão simples
  nativa com arredondamento para zero (exata bit a bit; `SPU_FP_NATIVE=0`
  desliga), um `goto` local para os desvios calculados dentro de uma função
  (`SPU_LOCAL_IB=0`), chamadas `SPU_DIRECT` e retornos de chamada mais
  baratos, medidos em cerca de +5% (32,2/33,0 fps ligado contra 30,7
  desligado, com ruído). A campanha agora chega ao limite do próprio jogo,
  ~30–33 fps; o tempo "ocupado" que resta na PPU é em boa parte o limitador
  de quadros do jogo. Os jobs SPU mais pesados rodam numa SPU só porque o
  jogo os prende nela (workload 5, contention 1..1), igual ao hardware.
- **Os ajustes do inFAMOUS 2 (2026-10-23, precisa de um teste jogando):**
  teclado e mouse no macOS (a janela os registra para o cellPad do jogo; um
  layout de FPS por padrão, remapeado com `PS3_INPUT_MAP=<input.toml>`, o
  formato do launcher do inFAMOUS 2; velocidade, inversão, zona morta e
  captura do mouse; `PS3_INPUT_MODE`), vibração (`PS3_RUMBLE`,
  `PS3_RUMBLE_STRENGTH`), modos de janela, escolha da tela, vsync e MetalFX
  na janela (que agora põe faixas pretas), SMAA (shaders MIT em
  `third_party/smaa`) e os filtros e o LUT da passada de apresentação,
  pacotes de texturas (`PS3_MODS`, `PS3_MODS_DUMP`) e a resolução interna
  (`PS3_RES_SCALE`, experimental). Os testes do toolkit passam; nada disso
  foi verificado numa sessão de jogo ainda. O launcher os expõe nas páginas
  Controles e Gráficos e passa cada um só quando é diferente do padrão.
- **Continuar a partir de um checkpoint (2026-10-23):** travava ao carregar,
  na v1.7 e na 01.00. O segundo programa de SPU raw tem uma área de
  parâmetros de 0x50 bytes (01.00: imagem+0x5680 de 0x56D0 bytes; v1.7:
  imagem+0x5780 de 0x57D0) que o PPU preenche antes de cada carga (ponteiros
  do heap, valores da fase, a janela MMIO da própria SPU raw), então a
  sequência de GETs que o `spu_raw.c` identifica pela fingerprint muda com a
  fase, o checkpoint e o número da SPU: um Continuar a partir do checkpoint
  do jogador chegava como `5771C48D28664765` (v1.7) / `78DE17DD3D838B86`
  (01.00), que não estava registrado. Agora o runtime também reconhece a
  sequência com a área de parâmetros de uma imagem registrada zerada
  (`spu_workload_register_img_masked`, patch do toolkit, upstream #538); o
  port registra `1894F66385AFD052` simples e `2B2DB6A5DE233A5F` mascarado
  (01.00), `6D03497950639952` simples e `F9726FDA4A599084` mascarado (v1.7).
  Verificado: o Continuar carrega e joga nas duas; campanha nova e
  renascimento continuam funcionando.
- **Som dos vídeos (2026-10-23):** o cellAdec era um stub que devolvia
  silêncio, então todo vídeo era mudo. Agora ele decodifica AC3 5.1 pelo
  FFmpeg carregado em tempo de execução (`libs/codec/adec_ffmpeg.c`; a fila de
  PCM, as informações do bitstream e os timestamps no `cellAdec.c`;
  `test_adec_decode.c`). O mixer POSIX se cadencia pelo relógio: um bloco a
  cada 5,333 ms, ajustes de no máximo um quarto de período, 4 ms de
  tolerância para um produtor atrasado, 7.1 reduzido para estéreo sem
  clipping, o nível inicial de cada porta respeitado. `PS3_AUDIO_DUMP=<wav>`
  grava a mistura e `PS3_AUDIO_STATS=1` registra o nível de cada porta.
  Verificado: a porta do vídeo de abertura a −25…−34 dBFS nos seis canais,
  0–0,2% de blocos em silêncio (era mudo); som presente no gameplay.
- **Névoa, fumaça e a água falsa (2026-10-23):** o `WPOS` segue o
  `SET_SHADER_WINDOW`, cuja origem aqui é embaixo à esquerda, e o
  descompilador de fragment programs o ignorava. A névoa no chão, a fumaça e
  mais um shader consultam o buffer de profundidade em 720 − `WPOS.y`, então
  o liam de cabeça para baixo, e a fase do rio mostrava um "reflexo" falso com
  listras verticais sobre o leito seco do riacho (cobrindo um corpo e uma
  caixa de munição). `rsx_fp_reads_wpos` / `rsx_fp_apply_wpos_window` aplicam
  a janela nos programas que leem o `WPOS`, e a janela do shader entra na
  chave do pipeline deles (`test_fp_decompiler`, 112 passam;
  `test_rsx_draw_engine`). Os shadow maps de profundidade de 16 bits em ordem
  swizzled (Morton) agora são gravados de volta na memória do guest, então os
  soldados distantes não ficam mais sobre retângulos escuros. Verificado nas
  duas campanhas.
- **Uma janela escondida (2026-10-23):** uma janela em tela cheia no próprio
  Space para de apresentar enquanto outro app está na frente; é o macOS, não
  um travamento. A janela agora registra isso no log ("window off screen…",
  `hidden=1` no `[RSX FPS]`) e mostra o quadro mais novo na volta.
- **O pool de shadow maps (2026-10-24):** retângulos escuros de bordas duras
  no chão. O draw engine guarda uma tabela de superfícies de cor e uma de
  superfícies de profundidade (zeta), 64 vagas cada, e nunca descartava nada.
  O Resistance empacota os shadow maps num pool por quadro cujos endereços
  mudam de um quadro para outro, então as tabelas enchiam, uma passada de
  sombra nova não achava vaga e era descartada sem aviso, e quem recebe a
  sombra lia um mapa vazio. As tabelas agora descartam a entrada usada há mais
  tempo (`libs/video/rsx_draw_engine.c`; o log diz "surface table full: …
  takes the slot of …"; `test_surface_tables_full`). Os descartes acontecem
  no log de uma sessão de jogo; a cena em si ainda precisa de um teste
  jogando.
- **Cadência a 60 fps (2026-10-24):** sem limite, a campanha faz um flip a
  cada tick de vblank que recebe. No relógio de 62,5 Hz (ticks de 16 ms) um
  quadro que leva 17 ms espera o tick seguinte ao próximo e fica na tela por
  32 ms, o que segurava o destravamento em ~50–56 fps. Com `RES_UNLOCK_FPS=1`
  o `port/patches.cpp` agora põe o vblank em 120 Hz (`PS3_VBLANK_HZ=120`, a
  não ser que já esteja definido): 88,5 fps medidos no gameplay, velocidade
  do jogo inalterada (o patch passa o jogo para passos de tempo reais).
  `PS3_FLIP_MAX_HZ=<hz>` (toolkit, desligado por padrão) mantém os flips a no
  mínimo 1/hz um do outro; limitar a 60 mediu pior (50 fps), porque o jogo
  espera o flip anterior antes do próximo quadro.
- **Polygon offset, posições invariantes, contagens de oclusão
  (2026-10-24):** o draw engine ignorava o polygon offset, que o jogo usa nos
  shadow maps dos personagens, nos decalques do terreno e nas sombras
  projetadas; agora ele vai para o Metal (`PS3RECOMP_ENG_POLY_OFFSET=0`
  desliga). A posição do vértice é compilada como `invariant`, então a
  pré-passada de profundidade da vegetação e a passada de cor com `EQUAL`
  calculam a mesma profundidade (`PS3RECOMP_MSL_INVARIANT=0` desliga). As
  contagens reais de oclusão ZPASS estão implementadas (visibility results do
  Metal; `test_zpass_engine`, `test_metal_occlusion`), mas desligadas por
  padrão, `PS3RECOMP_ENG_ZPASS=1` liga: esperar as contagens custou 84 → 50
  fps no modo 60 fps, um run ficou se arrastando, e nenhum ganho visível foi
  encontrado.
- **Tela dividida cooperativa (2026-10-24):** a visão do jogador 1 (esquerda)
  ficava preta. O `CLEAR_SURFACE` ignorava o scissor, então a limpeza do
  jogador 2 apagava a metade do jogador 1. As limpezas agora passam por
  ganchos novos do backend (`clear_color_rect`, `clear_depth_stencil_rect`;
  passadas de limpeza parcial no Metal) quando o scissor é mais estreito que
  o clip; um limite só de clip continua sendo uma limpeza inteira (deixar
  parcial a limpeza por quadro da composição custava uma passada a cada
  quadro). `PS3RECOMP_ENG_RECT_CLEAR=0` volta às limpezas inteiras;
  `test_metal_rect_clears`. Verificado no jogo: as duas metades aparecem.
- **Robustez (2026-10-24):** mais de duas horas de jogo roteirizado sem um
  crash. Multiplayer → Online Game travava offline; o diálogo de início de
  rede agora responde "cancelado" quando não há rede, e
  `sceNpManagerRequestTicket`/`RequestTicket2`/`GetTicket`/`GetTicketParam`
  foram adicionados (`test_netstart_offline`, `test_np_ticket_offline`). O
  NID desconhecido que o jogo chamava ao sair (`0x76948BFC`) é o
  `cellSysconfAbort`, adicionado junto com o `cellSysconfOpen`
  (`test_sysconf`); `cellPadInfoSensorMode` e `cellPadSetSensorMode` entraram
  (`test_pad_sensor_mode`). Um run completo agora não tem NIDs sem resolver.
  Verificado: todos os menus e extras do EBOOT, multiplayer em tela dividida
  offline, pausa e opções, Salvar e Sair → Continuar (exitspawn nos dois
  sentidos), morrer e renascer a 30 e 60 fps, um HDD novo e vazio, um
  `quick.sav` corrompido, 25 minutos seguidos a 60 fps e 22 minutos parado.
- **O vblank de 120 Hz fora do ambiente (depois da v0.2.1, na main):** o
  `port/patches.cpp` punha `PS3_VBLANK_HZ=120` com `setenv` no processo da
  campanha, e o exitspawn é um `execv`, então o Salvar e Sair o levava de volta
  ao front end: os passos fixos de 1/60 dos menus (`RES_MENU_60FPS`) podiam
  então rodar em velocidade dobrada com ticks de 120 Hz. O patch agora passa a
  taxa ao relógio de quadros do `port/main.cpp` por `g_res_vblank_hz`; o
  `PS3_VBLANK_HZ` ainda tem prioridade.

---

## O que falta para chegar ao gameplay

### Bloqueios

- [x] **Jobs de SPU estouram o tempo no gameplay.** Corrigido em 2026-10-22: os 15 módulos de job WWS traduzidos, seleção no estilo do kernel SPURS e eventos de SPU atômicos (veja "Gameplay" em Feito). O job manager carrega o código dos jobs na LS (por volta de 0x7C00–0x14000 da imagem dele) e o interpretador roda a uns 8–9 M passos/s. Os jobs que o primeiro frame precisa não cumprem os timeouts de 2 s do jogo. Capturar o código de cada job quando ele chega por DMA, traduzir (os `lift_spu_jobs.sh` / `relift_jobmod.py` do inFAMOUS 2 são o modelo) e registrar.
- [x] **Travamento depois do vídeo de abertura (intermitente).** Corrigido em 2026-10-22: pilha de callback HLE→jogo compartilhada (veja "Gameplay" em Feito). Uma thread do player de vídeo (`func_005F0360 → … → func_005EEFE4`) fica em `sys_mutex_lock`, o player nunca avisa que terminou e a thread principal fica no laço de carregamento (`func_00044DE0`, estado 2). `MUTEX_WATCH=2` mantém o lock bloqueante e informa quem espera há mais de 5 s e o dono do mutex. Até agora o travamento não se reproduziu com nenhum modo de watch. A política de mutex fair-share no macOS não resolveu.
- [x] **O jogador renasce acima do mapa.** Corrigido em 2026-10-22: uma parada e um reinício de SPU raw que o runtime descartava (veja "Estado atual"). Depois de morrer, a recarga do checkpoint mandava o jogador direto para o céu (~30–40 draws: céu, terreno distante e água); o save estava certo. Num teste com 12 mortes e renascimentos a câmera voltou sempre à posição da primeira vida (0 de 666 amostras da câmera fora da faixa, contra 193 de 292 antes); um run separado confirmou (de volta ao chão no mesmo lugar, andando normalmente).
- [x] **Retângulos escuros no chão (sombras) e a fumaça.** Corrigido em 2026-10-22, sumiram no jogo (conferido visualmente). A comparação de sombra das texturas estava na ordem do ARB; o console e o RPCS3 comparam profundidade gravada OP referência (a issue #19391 do RPCS3 mostra os mesmos retângulos). Corrigido junto com as outras correções da comparação com o RPCS3 (texturas de profundidade replicam a profundidade em todas as lanes, para partículas suaves e fumaça; snapshots de profundidade recortados à região da textura).
- [x] **Taxa de quadros.** ~30 fps no gameplay (2026-10-22): 32,5 fps com o pacote do inFAMOUS 2 contra 6,0 fps com as chaves dele desligadas; 30,5 fps na cidade com todas as correções.
- [x] **Prédios e telhados aparecem em azul forte chapado.** Corrigido em 2026-10-22: o wrap de 28 bits do offset de vértice (veja "Estado atual"); a cidade aparece com as cores certas.
- [x] **Texto das telas legais sobreposto.** Corrigido em 2026-10-23: o runtime informava a área de licença de um console japonês, então o jogo desenhava as telas legais da região errada umas sobre as outras. Agora informa América do Norte (upstream [#429](https://github.com/sp00nznet/ps3recomp/pull/429); `PS3_LICENSE_AREA` a troca); o boot vai dos logos ao título "Press the START button".
- [x] **Continuar a partir de um checkpoint travava ao carregar** (v1.7 e 01.00). Corrigido em 2026-10-23: a área de parâmetros do segundo programa de SPU raw é mascarada na fingerprint dele (veja "Estado atual"); o Continuar a partir do checkpoint do jogador carrega e joga nas duas campanhas.
- [x] **Todo vídeo era mudo.** Corrigido em 2026-10-23: o cellAdec decodifica AC3 5.1 pelo FFmpeg e o mixer POSIX se cadencia pelo relógio; o vídeo de abertura a −25…−34 dBFS nos seis canais.
- [x] **Água falsa, névoa e fumaça na fase do rio.** Corrigido em 2026-10-23: o `WPOS` com a origem embaixo à esquerda da janela do shader; a fase mostra o leito seco do riacho com névoa suave (nas duas campanhas).
- [x] **Soldados distantes sobre retângulos escuros.** Corrigido em 2026-10-23: shadow maps de profundidade de 16 bits swizzled gravados de volta na memória do guest.
- [x] **Retângulos escuros de bordas duras no chão (o pool de shadow maps).** Corrigido em 2026-10-24: as tabelas de superfícies e de profundidade do draw engine descartam a entrada usada há mais tempo em vez de largar as passadas de sombra novas quando enchem (veja "Estado atual"); os descartes aparecem no log de uma sessão de jogo, a cena ainda precisa de um teste jogando.
- [x] **Multiplayer → Online Game travava offline.** Corrigido em 2026-10-24: o diálogo de início de rede responde "cancelado" sem rede; chamadas de ticket do NP adicionadas.
- [ ] As lanes ociosas do job manager são paradas depois de 30 s fazendo polling da mesma linha da fila (`SPU_DMA_REPEAT_SECS`); conferir se elas voltam quando chega trabalho.

### Rumo ao primeiro capítulo

- [x] O vídeo de abertura da fase 1, assistido até o fim (5.584 imagens).
- [x] Tela de carregamento do capítulo 1 ("The Gauntlet", York) e "Press the START button".
- [x] Os primeiros frames de gameplay em York (primeira pessoa, HUD).
- [x] York jogado com controle: inimigos e aliados, tiros, morrer e renascer (2026-10-22).
- [x] Menu de pausa e opções dentro do jogo (2026-10-24, jogo roteirizado).

### Jobs de SPU

- [ ] Um jeito de capturar o código dos jobs do job manager (o próprio PM faz o DMA, então o `SPU_DUMP_MISS` não os vê).
- [ ] Um script que traduz todos os jobs capturados de uma vez.
- [ ] Os jobs de áudio do NextSynth, que rodam no interpretador desde o boot.

### Vídeo e áudio

- [ ] Sincronia de áudio e vídeo num vídeo longo.
- [x] Som dos vídeos: o cellAdec decodifica as faixas AC3 5.1 pelo FFmpeg (devolvia silêncio) (2026-10-23).
- [ ] A música do menu às vezes tem conteúdo a mais no centro/LFE (+4–6 dB); vem do sintetizador do jogo e varia de uma execução para outra.
- [x] Ritmo do áudio no macOS: o mixer alternava entre 2,4× e 0,95× do tempo real; agora segue o tempo real (do trabalho no inFAMOUS 2, 2026-10-23). Volume, sem som e saída: `PS3_AUDIO_VOLUME`, `PS3_AUDIO_MUTE`, `PS3_AUDIO_DEVICE`.
- [ ] Conferir o áudio de ouvido numa sessão longa em janela (`AUDIO_WAV` para gravar).

### Menu

- [x] Tela de título e menu principal.
- [x] Saves: `PROFILE.SAV` e `quick.sav` gravados e lidos em `PS3_HDD0_ROOT`.
- [x] Todos os menus e extras do EBOOT, e as Opções (2026-10-24, jogo roteirizado).
- [ ] `cellOskDialog` (teclado na tela), `cellMsgDialog`.

---

## O que falta para ficar 100% jogável

### Gameplay

- [x] O primeiro capítulo (York) com controle, em janela (2026-10-22; com os erros de renderização acima).
- [ ] O primeiro capítulo (York) com teclado e mouse (portados em 2026-10-23, precisa de um teste jogando).
- [ ] Zerar a campanha anotando o que quebra: todos os capítulos, os veículos, os chefes, o tiro alternativo de cada arma.
- [ ] **Em andamento (2026-10-24):** uma varredura de todas as fases da campanha: carregar cada uma, conferir renderização, jobs de SPU, chamadas HLE e fps, e corrigir o que quebrar.
- [ ] Uma ferramenta para carregar qualquer fase direto (a partir do método da varredura), para conferir e reproduzir bugs sem jogar até lá.
- [x] Tela dividida offline (cooperativo e versus): as duas viewports aparecem (2026-10-24; a metade do jogador 1 ficava preta, veja Gráficos). Ainda falta uma sessão com dois controles físicos.

### Gráficos

- [ ] Comparar com o original (RPCS3 ou o console) capítulo por capítulo; abrir um item por bug com captura, como no inFAMOUS 2.
- [x] A água falsa, a névoa no chão e a fumaça da fase do rio (`WPOS` e a janela do shader) e as sombras dos soldados distantes (2026-10-23).
- [x] Retângulos escuros de bordas duras no chão: as tabelas de superfícies e de profundidade descartam a entrada usada há mais tempo, então as passadas do pool de shadow maps não se perdem mais (2026-10-24; falta o teste jogando na cena).
- [x] Tela dividida cooperativa: a visão do jogador 1 ficava preta; as limpezas respeitam o scissor (`PS3RECOMP_ENG_RECT_CLEAR=0` reverte) (2026-10-24).
- [x] O polygon offset (shadow maps dos personagens, decalques do terreno, sombras projetadas; `PS3RECOMP_ENG_POLY_OFFSET=0` desliga) e uma posição de vértice invariante (a pré-passada de profundidade da vegetação e a passada de cor com `EQUAL`; `PS3RECOMP_MSL_INVARIANT=0` desliga) (2026-10-24).
- [ ] Pontos escuros na vegetação: parece ser o conteúdo do próprio jogo (o alpha granulado das folhas); confirmar com o RPCS3.
- [ ] Um defeito nos olhos de um personagem, relatado mas não reproduzido: precisa de um jeito de reproduzir.

### Desempenho

- [x] ~30 fps (a meta do próprio jogo) no gameplay: 30,5 fps na cidade com todas as correções (2026-10-22). Continuar medindo a cada mudança.
- [x] O cache de vértices entre frames do port do inFAMOUS 2: 5,9 → 14 fps (2026-10-22; `PS3RECOMP_ENG_VCACHE=0` desliga).
- [x] Builds otimizados: runtime -O2 e código PPU traduzido -O1 em toda configuração menos Debug (era -O0) (2026-10-22).
- [x] Um A/B de fps do pacote de desempenho do inFAMOUS 2: 32,5 fps ligado contra 6,0 fps com as chaves dele desligadas (`PS3RECOMP_ENG_VCACHE_SPLIT=0`, `PS3RECOMP_METAL_BIND_ALL=1`, `PS3_FRAMECLOCK_US=4000`), mesmo build, primeiros 60 s de gameplay (2026-10-22).
- [x] A campanha no limite do próprio jogo, ~30–33 fps (2026-10-23): a resolução da profundidade como RGBA8 na GPU (thread do RSX de 54% → 12% ocupada, reenvios de textura a cada 120 quadros de ~100 → 0), o caminho rápido do store-watch da PPU (era 28% da thread principal), relógios do guest pelo `mach_absolute_time`, e na SPU precisão simples nativa, `goto` local para desvios calculados, `SPU_DIRECT` e retornos de chamada mais baratos (cerca de +5%, 32,2/33,0 contra 30,7 fps, com ruído). O que sobra do tempo "ocupado" da PPU é em boa parte o limitador de quadros do jogo; o jogo prende os jobs SPU mais pesados numa SPU só (workload 5, contention 1..1), como no hardware.
- [x] Cadência a 60 fps: com `RES_UNLOCK_FPS=1` o vblank roda a 120 Hz; 88,5 fps no gameplay (eram ~50–56), velocidade do jogo inalterada (2026-10-24). O `PS3_FLIP_MAX_HZ` limita os flips (desligado por padrão; 60 mediu 50 fps).
- [ ] Contagens reais de oclusão ZPASS (`PS3RECOMP_ENG_ZPASS=1`): implementadas, desligadas por padrão; esperar as contagens custou 84 → 50 fps a 60 fps e nenhum ganho visível foi encontrado. Ligar só se uma cena precisar.
- [ ] Uma medição de fps de verdade da campanha v1.7 (o mesmo A/B da 01.00).
- [ ] Medir quanto do tempo de SPU é interpretado e cortar (traduzir o que sobrar).

### Estabilidade

- [x] Sessões longas sem crash (2026-10-24): mais de duas horas de jogo roteirizado, 25 minutos seguidos a 60 fps e 22 minutos parado.
- [ ] Um relatório de travamento no estilo do `watch_stall.sh`.
- [x] Morrer e renascer a 30 e 60 fps (`RES_UNLOCK_FPS=1`) (2026-10-24).
- [x] Salvar e Sair → Continuar (exitspawn nos dois sentidos), um HDD novo e vazio, um `quick.sav` corrompido (2026-10-24).
- [x] Salvar e Sair com a opção de 60 fps não leva mais o vblank de 120 Hz de volta aos menus (`g_res_vblank_hz` em vez do ambiente; depois da v0.2.1, na main).
- [x] O NID desconhecido `0x76948BFC` que o jogo chama ao sair é o `cellSysconfAbort`; adicionado junto com o `cellSysconfOpen` (2026-10-24). Um run completo não tem NIDs sem resolver.

### Controles

- [ ] Teclado e mouse no macOS com mapeamento configurável (`PS3_INPUT_MAP`): portados do trabalho do inFAMOUS 2 e expostos na página Controles do launcher (2026-10-23); os testes passam, precisa de um teste jogando.
- [x] DualSense/DualShock/Xbox via SDL2 (um DualSense jogou o capítulo 1, 2026-10-22).
- [ ] Um controle físico com o launcher: o processo do jogo fica em segundo plano, então `SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1` está ligado; ainda não testado com um controle.
- [ ] Vibração com um controle físico: o `game.self` da v1.7 chama `cellPadSetActDirect`; a vibração (`PS3_RUMBLE`, `PS3_RUMBLE_STRENGTH`) foi portada em 2026-10-23, precisa de um teste jogando.
- [ ] SIXAXIS: o jogo chama `cellPadSetSensorMode`/`cellPadInfoSensorMode` (os dois implementados em 2026-10-24); descobrir para que usa o movimento e mapear num botão.

### Atualização do jogo (v1.7)

O servidor de atualizações da Sony não lista pacotes para nenhum ID de título do Resistance: o jogo se atualizava pelos próprios servidores de multiplayer (desligados em 2014-04-08). A última versão é a v1.7 (Patch 7), `patch_01`…`patch_07.psarc` em `USRDIR/data`; o `patch_06` traz um `game.self` novo (`/data/new_game.self`, ~8,7 MB), dados de colisão novos para as fases 200–320, menus, fontes, traduções, vibração e suporte a map packs. As notas das atualizações são quase todas de multiplayer. O servidor de revival PSRewired (psrewired.com) serve o Resistance online e, segundo a wiki do RPCS3, a atualização v1.7.

Feito em 2026-10-22: a campanha roda a v1.7 (veja "Estado atual").

- [x] Traduzir de novo o `game.self` atualizado do `patch_06` (19.706 funções), os 13 módulos de job que mudaram e os dois programas das SPUs raw; `tools/v17/regen_v17.sh`.
- [x] Rodá-lo a partir dos menus da 01.00: o exitspawn escolhe o build v1.7; Nova campanha → abertura de York, gameplay e renascimento verificados.
- [x] Manter os arquivos da atualização fora da pasta de jogo (o EBOOT 01.00 não roda com eles): o `tools/play.sh` e o launcher se recusam a começar.
- [ ] Menus da v1.7: precisam de um `EBOOT.BIN` v1.7 que não seja NPDRM, e ele parece não existir (o EBOOT da atualização só existe em NPDRM); provavelmente impossível.
- [ ] Vibração, uma sessão longa e uma medição de fps na v1.7 (veja Controles, Estabilidade, Desempenho).

### Rebuild

- [x] Um script para o rebuild inteiro a partir do disco do próprio jogador (`tools/rebuild.sh`).
- [x] Os programas das SPUs raw e os jobs de SPU a partir do disco, sem um run de captura: os módulos de job e os programas das SPUs raw são recortados dos ELFs (`tools/regen_jobmods.sh`, `tools/extract_rawspu_elf.py`, 2026-10-23).
- [ ] Descriptografia sem RPCS3 (o `decrypt_self.py` do inFAMOUS 2, com o arquivo de chaves do próprio jogador).

---

## Depois de jogável (melhorias)

### Gráficos e resolução

- [ ] Resolução interna acima de 720p (`PS3_RES_SCALE`, experimental), MetalFX (`PS3_UPSCALE=metalfx`), tela cheia/sem borda (`PS3_WINDOW_MODE`, `PS3_DISPLAY`, `PS3_VSYNC`), SMAA e filtros, pacotes de texturas: portados do trabalho do inFAMOUS 2 (2026-10-23), os testes passam, precisa de um teste jogando.
- [x] Acima de 30 fps e um campo de visão maior (2026-10-23): os patches da comunidade para o RPCS3 (Whatcookie, FlexBy, illusion, Score) trazidos para a recompilação estática. A metade de dados é gravada depois que o ELF carrega; a metade de código é traduzida nas duas formas (`ppu_lifter.py --insn-patches`) e escolhida em tempo de execução, e cada conjunto só se aplica quando 7–10 palavras conhecidas batem com o ELF em execução. `RES_UNLOCK_FPS=1`: ~56 fps na campanha com a máquina ociosa, 01.00 e v1.7, velocidade do jogo normal no tempo real (o movimento de câmera da introdução leva 14,4–16,4 s a 33, 56 e 100 fps; andar ~6,3 unidades/s); `PS3_VBLANK_HZ=120` ~100 fps. `RES_MENU_60FPS=1`: menus de 32 → 59 fps. `RES_FOV=<0.1..4>` (o do jogo é 1,0; o "Extended FOV" do RPCS3 é 0,7). O botão "60 fps" do launcher passa os dois patches de fps. Desde 2026-10-24 o destravamento roda o vblank a 120 Hz: 88,5 fps no gameplay.

### App para instalar e jogar (launcher)

- [x] A janela mantém posição, tamanho e tela cheia no exitspawn do menu para a campanha (`PS3RECOMP_WINDOW_STATE`, 2026-10-22).
- [x] Uma janela única e persistente (2026-10-22): o launcher `Resistance` (`tools/host_window/host_window.m`) é dono da janela e os executáveis do menu e da campanha desenham em IOSurfaces compartilhadas com ele; o jogo não tem janela nem ícone no Dock próprios. Fechar a janela encerra o jogo; se o launcher morre, o jogo sai em ~2 s; sem ele o backend abre a própria janela. Verificado dos menus até a abertura da campanha a ~33 fps.
- [x] O "guest FPS" no título da janela conta cada flip (mostrava 0 com a janela do launcher) (2026-10-22).
- [x] O app launcher (`launcher/`, Tauri, 2026-10-22): escolher e montar o ISO, pasta de jogo, detecção da pasta do port, versão da campanha (v1.7/01.00), ajustes (gráficos, desempenho, jogo), um gerenciador de saves (aqui e no RPCS3), o log do jogo, modo de depuração, verificação de atualizações, atalho na Mesa e na Steam, português do Brasil e inglês. O Parar funciona: a janela agora sai quando o jogo sai.
- [x] Página Controles (associações de teclado, mouse e controle gravadas em `input.toml`, o teste do controle, o modo de entrada, mouse, vibração) e ajustes de Gráficos (modo de janela, tela, vsync, MetalFX, resolução interna, SMAA, filtros e LUT, pacotes de texturas e a extração deles), em português do Brasil e inglês; cada ajuste passado só quando é diferente do padrão (2026-10-23; o lado do jogo precisa de um teste jogando).
- [x] Release v0.1.0 (2026-10-23, pré-release): o launcher para macOS e uma prévia para Windows.
- [x] Release v0.2.0 (2026-10-23, pré-release): o launcher com os ajustes de Controles, Gráficos e Áudio, 60 fps e FOV.
- [x] Release v0.2.1 (2026-10-24, pré-release): as correções da cadência a 60 fps, da tela dividida e das sombras; o botão "60 fps" do launcher recebe o vblank de 120 Hz do port.
- [x] Ajustes de áudio (volume, sem som, saída) e um botão "60 fps" só (2026-10-23).
- [ ] Um controle físico com o launcher (veja Controles).
- [ ] Ícone a partir do `ICON0.PNG` do próprio jogador, app bundle com Game Mode no macOS.

### Outras plataformas

- [ ] Linux (Vulkan) e Windows (D3D12), com o launcher e o rebuild rodando nativamente. O carregamento do XInput em tempo de execução no Windows (upstream #494) está no patch do toolkit, sem teste no Windows; a nova cadência do mixer de áudio também.

### Fora do escopo por enquanto

- [ ] Online (partidas de 40 jogadores, `sceNp`, `sys_net`, chat de voz via `cellMic`): os servidores oficiais acabaram; só com um servidor da comunidade. Multiplayer → Online Game agora volta atrás offline em vez de travar (2026-10-24).

---

## Feito e concluído

### Boot e pipeline

- [x] Dump do disco `BCUS98107` v01.00, montado como raiz do VFS.
- [x] `EBOOT.BIN` e `game.self` descriptografados, analisados (EBOOT com 25.022 funções e 183 imports; `game.self` com 18.994 funções e 134 imports) e traduzidos; os dois compilam no macOS/arm64 com Metal.
- [x] Policy module do job manager traduzido na base LS 0xA00 (imagem SPU 1).
- [x] Programas das SPUs raw `1894F66385AFD052` e `2B2DB6A5DE233A5F` (área de parâmetros mascarada) traduzidos na base LS 0x60 (imagens SPU 2 e 3).

### Front end

- [x] CRT e TLS, `sys_memory`, `cellSysmodule`, `cellDiscGameGetBootDiscInfo`, `cellSysCacheMount`, `cellGameDataCheckCreate2`.
- [x] `cellSpursInitialize`, os workloads do job manager, inicialização do `cellGcm` e flips na tela.
- [x] Jogo fora da thread principal do macOS.
- [x] Telas legais, instalação no HDD da primeira execução, tela de título, menu principal, perguntas de dificuldade e de inverter o Y, avisos de sobrescrever.
- [x] `PROFILE.SAV` e `quick.sav` gravados e lidos.

### Início da campanha

- [x] `sys_game_process_exitspawn` EBOOT → `game.self`, com o bloco de dados onde o `game.self` o procura.
- [x] `sys_ppu_thread_get_stack_information` para a thread principal (antes devolvia 0 + 0, e o primeiro boot lia 0xFFFFF000).
- [x] Vídeos na tela: FFmpeg em tempo de execução, vértices imediatos `DATA2S`, dreno por ociosidade da última imagem reordenada.
- [x] SPUs raw no macOS: thread POSIX, arena do guest com os 32 bits inteiros, um programa por SPU.
- [x] O capítulo 1 carrega até "Press the START button".

### Gameplay (2026-10-22)

- [x] Travamento depois do vídeo de abertura: todo callback HLE→jogo rodava numa pilha de guest compartilhada (0xCFFE0000), e os callbacks de adec e vdec do player escreviam um por cima do outro; um guard de mutex então destravava NULL e o mutex do player ficava travado. Pilhas de callback por thread do host (patch do toolkit). 3 de 3 runs passam; antes 5 de 6 travavam.
- [x] Os 15 módulos de job WWS (`C0DEC0DE`) embutidos nos executáveis, traduzidos na própria base (13 em 0x4000, dois de áudio em 0x7000; `tools/regen_jobmods.sh`).
- [x] Seleção de workloads no estilo do kernel SPURS (`SPURS_RPCS3_SCHED`, que o port liga): o poll do policy module recebe o workload que o kernel rodaria, o readyCount é do PM, as prioridades contam. Sem isso os jobs de gameplay (`igGeomCullClip`, `SpuBucketerDrawSort`) estouravam o tempo.
- [x] Eventos de SPU atômicos: o aviso de reserva perdida vindo de uma escrita do PPU apagava o evento de stall-and-notify da lista do job manager, que esperava a interrupção para sempre; o primeiro frame de gameplay nunca terminava.
- [x] `sys_mutex_destroy` recusa um mutex travado ou com quem espera (EBUSY), como o lv2.
- [x] Capítulo 1 jogado numa janela com um DualSense: primeira pessoa, HUD, inimigos e aliados, tiros, morrer e renascer.
- [x] `tools/play.sh`: jogar em janela com controle; saves, cache da instalação e log em `~/Library/Application Support/resistance-recomp/`, taxa de quadros no título da janela.
- [x] Perfil do host de uma sessão em janela: a decodificação de vértices na CPU da thread do RSX é o gargalo.
- [x] O log `[spurs-pm]` de cada poll do job manager só com `SPURS_PM_TRACE` (gravava 1,9 milhão de linhas numa sessão curta, o bastante para custar taxa de quadros).

### Renderização e desempenho (2026-10-22)

- [x] O mundo azul: offsets de array de vértices somados ao `VERTEX_DATA_BASE_OFFSET` fazem wrap em 28 bits (`rsx_dsp_resolve_vertex_offset`, os dois caminhos de fetch, teste de regressão). A cidade aparece com as cores certas.
- [x] Cache de vértices entre frames do port do inFAMOUS 2: 5,9 → 14 fps no gameplay (mesmo build -O0, cache ligado vs desligado).
- [x] Builds otimizados: o port vinha sendo compilado em -O0; agora o runtime é -O2 e o código PPU traduzido -O1 em toda configuração menos Debug.
- [x] Correções de uma comparação com o RPCS3 (todos os testes de vídeo passam; o efeito no jogo está sendo avaliado): direção da comparação das sombras (profundidade gravada OP referência), snapshots de profundidade recortados ao tamanho da textura e liberados (vazavam a cada frame), texturas de profundidade replicam a profundidade em todas as lanes, texturas comprimidas nunca são resolvidas a partir de render targets, threads PPU começam com VSCR[NJ]=1, sem `getenv` a cada escrita em canal MFC da SPU.
- [x] O pacote de desempenho e correções do inFAMOUS 2 (testes passam; A/B de fps: 32,5 contra 6,0 fps): cache de vértices por stream (streams de malhas estáticas guardados à parte do stream de iluminação de cada frame; `PS3RECOMP_ENG_VCACHE_SPLIT=0` desliga), ring de staging do Metal, bind só do estado que mudou (`PS3RECOMP_METAL_BIND_ALL=1` desliga), memo de programas VP/FP e índice de hash de pipelines, decodificação de índices, cache do `mfc_for` da SPU, lock TTAS da lock line, `cflts`/`cfltu` em NEON, espera de fila de eventos da SPU cheia, cor de borda das texturas, lanes de profundidade X32_FLOAT, snapshots de profundidade aposentados por escritas de profundidade, cache de shaders MSL (`PS3_SHADER_CACHE=<pasta>`, `PS3_SHADER_ASYNC=1` opcional), relógio de frame de 1 ms e um present por flip (`PS3_FRAMECLOCK_US=4000` é o ritmo antigo).
- [x] O executável se chama `ResistanceRecomp` (era `MyGameRecomp`), e a janela mantém posição, tamanho e tela cheia no exitspawn.
- [x] Diagnóstico do renascimento: o save do checkpoint bate com o do RPCS3 com diferença de 166 bytes; o bug não estava no save.
- [x] Renascimento corrigido: `SPU_RunCntl`=0 para de fato uma SPU raw e `SPU_RunCntl`=1 a começa do zero (no modelo do `RawSPUThread.cpp` do RPCS3); 12 mortes num teste, a câmera de volta à posição da primeira vida todas as vezes.
- [x] Uma janela persistente pelo launcher `Resistance` e IOSurfaces compartilhadas (~0,35 ms de GPU por frame, sem cópia na CPU); a janela sobrevive ao exitspawn.

### A campanha v1.7 e o launcher (2026-10-22)

- [x] O `game.self` da v1.7 (`/data/new_game.self` do `patch_06`) traduzido de novo e compilado como `build/BCUS98107_game_v17`; 13 dos 15 módulos de job e os dois programas das SPUs raw traduzidos de novo (`6D03497950639952`, `F9726FDA4A599084` mascarado); o policy module não mudou.
- [x] O exitspawn roda o build v1.7 quando ele e o ELF dele existem (`RES_GAME_VERSION=100` mantém a 01.00); menus → Nova campanha → abertura de York, gameplay e renascimento verificados.
- [x] `tools/v17/regen_v17.sh` (`tools/rebuild.sh v17`), `tools/v17/psarc_extract.py`; o `tools/play.sh` diz qual campanha roda e se recusa a começar com os `patch_NN.bin` da atualização na pasta de jogo.
- [x] Sombras e fumaça corrigidas no jogo (conferido visualmente).
- [x] O app launcher (`launcher/`), com o Parar funcionando (a janela recolhe o jogo de forma confiável).
- [x] O "guest FPS" do título conta cada flip.

### Desempenho e os ajustes do inFAMOUS 2 (2026-10-23)

- [x] Resolução da profundidade como RGBA8 na GPU: thread do RSX de 54% → 12% ocupada, reenvios de textura a cada 120 quadros de ~100 → 0.
- [x] Caminho rápido do store-watch da PPU, relógios do guest pelo `mach_absolute_time`; na SPU precisão simples nativa (exata bit a bit), `goto` local para desvios calculados, `SPU_DIRECT`, retornos de chamada mais baratos (cerca de +5%). A campanha no limite do próprio jogo, ~30–33 fps.
- [x] Teclado e mouse, vibração, modos de janela, vsync, MetalFX, SMAA, filtros e LUT, pacotes de texturas e a resolução interna portados do trabalho do inFAMOUS 2; os testes passam (falta o teste jogando, veja Controles e Gráficos).
- [x] A página Controles e os novos ajustes de Gráficos do launcher; v0.1.0 lançado.

### 60 fps, telas legais, áudio e correções do upstream (2026-10-23)

- [x] Os patches de 60 fps / FOV (`port/patches.cpp`, `port/patches/*.insn.json`; `tools/rebuild.sh lift` e `tools/v17/regen_v17.sh` passam `--insn-patches`, `tools/relift.sh` traduz de novo um lift antigo).
- [x] A sobreposição das telas legais (área de licença, #429).
- [x] De pull requests do upstream: os bits de desvio dos vertex programs (#485, mais uma correção do slot inicial do programa; 19 dos 343 vertex programs do Resistance desviam), `WPOS` em alvos com AA 2× (#446, com a resolução interna), a condição de corrida da tabela de fds e a troca leitura/escrita (#430, #456), a nova checagem do `stwcx.` (#522), decodificação rápida de ARGB linear (#432), a memória dos slots de textura (#438), o decrementador da SPU no relógio bruto (#451), ceder a vez em DMA de SPU repetido (#448), pequenas economias de DMA de SPU (#453–#455), `getenv` uma vez por ponto de chamada (#515), XInput carregado em tempo de execução no Windows (#494). Do trabalho no inFAMOUS 2: ritmo, volume e saída do áudio, o giro opcional do `usleep` (`PS3_USLEEP_SPIN_US`). Um run de regressão de gameplay com todas elas: renderiza, sem erros, renascer OK.

### Continuar, som dos vídeos, névoa e água (2026-10-23)

- [x] Programas de SPU raw com área de parâmetros: a fingerprint mascarada (`spu_workload_register_img_masked`, `test_spu_workload_masked.c`), os registros mascarados da 01.00 e da v1.7, o `tools/extract_rawspu_elf.py` (sem run de captura). O Continuar a partir de um checkpoint carrega e joga nas duas campanhas.
- [x] Som dos vídeos: a decodificação AC3 5.1 do cellAdec pelo FFmpeg (`libs/codec/adec_ffmpeg.c`, `test_adec_decode.c`) e a cadência do mixer POSIX (um bloco a cada 5,333 ms pelo relógio, 7.1 → estéreo sem clipping, o nível inicial de cada porta).
- [x] `WPOS` com a origem da janela do shader (a água falsa, a névoa no chão e a fumaça da fase do rio) e shadow maps de profundidade de 16 bits swizzled gravados na memória do guest (as sombras dos soldados distantes); verificado nas duas campanhas.
- [x] A janela avisa quando está escondida ou fora da tela e mostra o quadro mais novo na volta.

### Sombras, cadência a 60 fps, tela dividida e robustez (2026-10-24)

- [x] As tabelas de superfícies e de profundidade do draw engine descartam a entrada usada há mais tempo (o pool de shadow maps; "surface table full: … takes the slot of …"; `test_surface_tables_full`).
- [x] O vblank de 120 Hz com `RES_UNLOCK_FPS=1` (`port/patches.cpp`; 88,5 fps no gameplay) e o `PS3_FLIP_MAX_HZ` (desligado por padrão).
- [x] O polygon offset, uma posição de vértice invariante e as contagens de oclusão ZPASS (desligadas por padrão, `PS3RECOMP_ENG_ZPASS=1`).
- [x] Limpezas com scissor por `clear_color_rect` / `clear_depth_stencil_rect` (passadas de limpeza parcial no Metal): as duas metades da tela dividida aparecem.
- [x] Respostas offline para o diálogo de início de rede e os tickets do NP (o Online Game não trava mais), `cellSysconfAbort` / `cellSysconfOpen`, `cellPadInfoSensorMode` / `cellPadSetSensorMode`; um run completo não tem NIDs sem resolver. Mais de duas horas de jogo roteirizado sem crash.

### Ferramentas

- [x] `RES_PEEK` (leitura de memória do guest pelo `port/stubs.cpp`), `MUTEX_WATCH=1|2`, `VDEC_PICLOG`, `PPU_STACKINFO_LOG`, `SPU_RAW_DUMP_DIR`.
- [x] Runs headless com dump de frames e entrada roteirizada (`tools/run.sh`, `start.sh`, `press.sh`, `snap.sh`, `to_campaign.sh`).
- [x] `RSX_ENGINE_PASS_DUMP_TRIGGER=<arquivo>`: o dump por passada começa quando esse arquivo aparece, para um script capturar o frame até onde levou o jogo (2026-10-22).
- [x] Comandos de analógico no `PAD_FILE`, `S lx ly rx ry polls`: segurar os analógicos por script para andar ou virar a câmera (2026-10-22).
- [x] O `RSX_ENGINE_PASS_DUMP_DRAWS` registra o formato e os primeiros bytes dos atributos de vértice de cada draw; `RSX_VPC_LOG=<constante>[,<draw>]` registra uma constante do vertex program a cada 30 frames (`c16` = posição da câmera nos shaders do mundo) (2026-10-22).
- [x] `PS3_AUDIO_DUMP=<arquivo.wav>` grava a mistura final de uma sessão inteira; `PS3_AUDIO_STATS=1` registra a cada ~5 s o nível de cada porta de áudio e da mistura, os blocos em silêncio e o clipping (2026-10-23).
- [x] `tools/extract_rawspu_elf.py`: os programas das SPUs raw recortados do `game.elf`, com a área de parâmetros zerada e um manifesto JSON (2026-10-23).

---

## Retribuindo

Pull requests para o [sp00nznet/ps3recomp](https://github.com/sp00nznet/ps3recomp), a partir de branches no [erikomis/ps3recomp](https://github.com/erikomis/ps3recomp), cada um com teste:

- [ ] [#463](https://github.com/sp00nznet/ps3recomp/pull/463) Dispatch do RSX: toda codificação imediata do ATTR0 emite um vértice (os vídeos pretos).
- [ ] [#469](https://github.com/sp00nznet/ps3recomp/pull/469) `sys_ppu_thread_get_stack_information` para a thread principal.
- [ ] [#471](https://github.com/sp00nznet/ps3recomp/pull/471) cellVdec: dreno por ociosidade das imagens que o libavcodec segura.
- [ ] [#473](https://github.com/sp00nznet/ps3recomp/pull/473) SPUs raw no POSIX, um programa por SPU.
- [ ] [#474](https://github.com/sp00nznet/ps3recomp/pull/474) Diagnóstico `MUTEX_WATCH`.
- [ ] [#475](https://github.com/sp00nznet/ps3recomp/pull/475) Template: a arena do guest no POSIX cobre as janelas das SPUs raw.
- [ ] [#477](https://github.com/sp00nznet/ps3recomp/pull/477) Template: o jogo fora da thread principal do macOS.
- [ ] [#488](https://github.com/sp00nznet/ps3recomp/pull/488) lift_jobmods: cada módulo de job traduzido na base de link do cabeçalho.
- [ ] [#489](https://github.com/sp00nznet/ps3recomp/pull/489) Callbacks HLE→jogo numa pilha por thread do host (o travamento depois do vídeo).
- [ ] [#490](https://github.com/sp00nznet/ps3recomp/pull/490) `sys_mutex_destroy` devolve EBUSY para mutex travado ou com quem espera.
- [ ] [#491](https://github.com/sp00nznet/ps3recomp/pull/491) Diagnóstico `SPU_IRQWAIT_DIAG`.
- [ ] [#492](https://github.com/sp00nznet/ps3recomp/pull/492) Eventos de SPU atômicos (o primeiro frame de gameplay).
- [x] [#493](https://github.com/sp00nznet/ps3recomp/pull/493) Seleção no estilo do kernel SPURS: fechado; fica no patch do toolkit deste repositório.
- [ ] [#504](https://github.com/sp00nznet/ps3recomp/pull/504) Endereços de array de vértices fazem wrap em 28 bits (o mundo azul).
- [ ] [#505](https://github.com/sp00nznet/ps3recomp/pull/505) `SPU_RunCntl`=0 para uma SPU raw rodando (o renascimento; em cima do #473).
- [ ] [#506](https://github.com/sp00nznet/ps3recomp/pull/506) Otimizar um build configurado sem `CMAKE_BUILD_TYPE`.
- [ ] [#507](https://github.com/sp00nznet/ps3recomp/pull/507) Um snapshot de profundidade é a região que a textura indica (em cima do #502).
- [ ] [#508](https://github.com/sp00nznet/ps3recomp/pull/508) Uma textura comprimida nunca é a saída de um render target.
- [ ] [#509](https://github.com/sp00nznet/ps3recomp/pull/509) macOS: uma janela só através do exitspawn (`tools/host_window`, modo host-surface do Metal); complemento: a janela avisa quando está fora da tela e mostra o quadro mais novo na volta.
- [ ] [#538](https://github.com/sp00nznet/ps3recomp/pull/538) spu_workload: imagens de SPU raw com uma área de parâmetros que o carregador preenche (o travamento do Continuar; empilhado sobre o #473).
- [ ] [#539](https://github.com/sp00nznet/ps3recomp/pull/539) ppu_lifter: `--insn-patches` traduz instruções com patch como chaves em tempo de execução (os patches de 60 fps).
- [ ] [#540](https://github.com/sp00nznet/ps3recomp/pull/540) sys_timer: um relógio do host só para `mftb`, hora do sistema e timers (`mach_absolute_time` no macOS).
- [ ] [#560](https://github.com/sp00nznet/ps3recomp/pull/560) Um `CLEAR_SURFACE` com scissor limpa só o retângulo do scissor (tela dividida cooperativa) (2026-10-24).
- [ ] [#562](https://github.com/sp00nznet/ps3recomp/pull/562) O polygon offset chega ao Metal (2026-10-24).
- [ ] [#563](https://github.com/sp00nznet/ps3recomp/pull/563) Uma posição de vértice invariante na tradução para MSL (2026-10-24).
- [ ] [#564](https://github.com/sp00nznet/ps3recomp/pull/564) `cellSysconfOpen` / `cellSysconfAbort` (2026-10-24).
- [ ] [#565](https://github.com/sp00nznet/ps3recomp/pull/565) `cellPadInfoSensorMode` / `cellPadSetSensorMode` (2026-10-24).
- [x] Não abertos, já cobertos no upstream: o descarte LRU das superfícies/zeta (o #499 do n0madic já tem), o resultado offline do início de rede (já no #503 deste port), as chamadas de ticket do sceNp (#543; o `sceNpManagerRequestTicket` v1 ficou de fora). Segurados: o `PS3_FLIP_MAX_HZ` (mediu pior) e a contagem ZPASS (cara, sem ganho visível).
- [x] Repositório próprio (`resistance-recomp`): fontes do port, um patch do toolkit, `tools/setup_toolkit.sh`, README, créditos e um `.gitignore` que bloqueia dados do jogo, firmware e saves.
