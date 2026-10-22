# Roadmap

🌐 [English](ROADMAP.md) · **Português (BR)**

Estado do port nativo de Resistance: Fall of Man (`BCUS98107`, disco, v01.00)
para macOS/arm64 e o que vem a seguir. Mesmo formato do roadmap do inFAMOUS 2.
`[x]` = feito e verificado; `[ ]` = a fazer; "(em andamento)" = sendo feito.

## Resumo (2026-10-22)

- **Roda até o gameplay:** os dois executáveis estão traduzidos, compilam e rodam no macOS/arm64 com Metal. Boot, telas legais, instalação da primeira execução, tela de título e menu principal levam a uma campanha nova. O `game.self` então toca o vídeo de abertura da fase 1 e carrega o capítulo 1, "The Gauntlet" (York).
- **Jogado:** o capítulo 1 numa janela com um DualSense (SDL2), `tools/play.sh`: primeira pessoa, HUD, inimigos e aliados, tiros, morrer e renascer. A cidade aparece com as cores certas (o mundo azul foi corrigido).
- **Mais rápido:** o cache de vértices entre frames levou o gameplay de 5,9 para ~14 fps (mesmo build sem otimização). Depois disso o build passou a ser otimizado (runtime -O2, código traduzido -O1) e o pacote de desempenho do inFAMOUS 2 entrou; novas medições em andamento.
- **Lançado:** nada ainda.
- **Em aberto:** depois de morrer, o jogador renasce muito acima do mapa; retângulos escuros no chão (sombras) e a fumaça, a conferir no jogo depois das correções da comparação com o RPCS3; o texto das telas legais anteriores se sobrepõe às seguintes; teclado e mouse não funcionam no macOS (é preciso um controle); sem vibração; sem Windows nem Linux.
- **Próximo:** a posição no renascimento (prioridade máxima), conferir sombras e fumaça no jogo, medir o fps com todas as mudanças, depois o launcher de janela única e a atualização v1.7.

## Estado atual (2026-10-22)

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
  (patch do toolkit, `port/main.cpp`). Os dois programas são capturados com
  `SPU_RAW_DUMP_DIR` e traduzidos (`tools/regen_rawspu.sh`).
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
  -O1 em toda configuração menos Debug.

---

## O que falta para chegar ao gameplay

### Bloqueios

- [x] **Jobs de SPU estouram o tempo no gameplay.** Corrigido em 2026-10-22: os 15 módulos de job WWS traduzidos, seleção no estilo do kernel SPURS e eventos de SPU atômicos (veja "Gameplay" em Feito). O job manager carrega o código dos jobs na LS (por volta de 0x7C00–0x14000 da imagem dele) e o interpretador roda a uns 8–9 M passos/s. Os jobs que o primeiro frame precisa não cumprem os timeouts de 2 s do jogo. Capturar o código de cada job quando ele chega por DMA, traduzir (os `lift_spu_jobs.sh` / `relift_jobmod.py` do inFAMOUS 2 são o modelo) e registrar.
- [x] **Travamento depois do vídeo de abertura (intermitente).** Corrigido em 2026-10-22: pilha de callback HLE→jogo compartilhada (veja "Gameplay" em Feito). Uma thread do player de vídeo (`func_005F0360 → … → func_005EEFE4`) fica em `sys_mutex_lock`, o player nunca avisa que terminou e a thread principal fica no laço de carregamento (`func_00044DE0`, estado 2). `MUTEX_WATCH=2` mantém o lock bloqueante e informa quem espera há mais de 5 s e o dono do mutex. Até agora o travamento não se reproduziu com nenhum modo de watch. A política de mutex fair-share no macOS não resolveu.
- [ ] **O jogador renasce acima do mapa** (prioridade máxima, em andamento). Depois de morrer e renascer no checkpoint o jogador volta muito acima do mapa: o número de draws cai de ~250–500 para ~30–40 (só céu, terreno distante e água). O save do checkpoint (`USRDIR/quick.sav`, 52.832 bytes) bate com o do RPCS3 com diferença de 166 bytes, então o save está certo e o posicionamento no renascimento está errado. Hipóteses: física de PPU/SPU no recarregamento; sensibilidade ao tempo de frame (dt) depois de um carregamento longo (o próprio patch de 60 fps do RPCS3 precisa de um passo mínimo). `RSX_VPC_LOG=16` acompanha a posição da câmera.
- [ ] **Retângulos escuros no chão (sombras) e a fumaça** (em andamento). A comparação de sombra das texturas estava na ordem do ARB; o console e o RPCS3 comparam profundidade gravada OP referência (a issue #19391 do RPCS3 mostra os mesmos retângulos). Corrigido junto com as outras correções da comparação com o RPCS3 (texturas de profundidade replicam a profundidade em todas as lanes, para partículas suaves e fumaça); todos os testes de vídeo passam, falta conferir no jogo.
- [ ] **Taxa de quadros** (em andamento). ~14 fps no gameplay com o cache de vértices, medidos antes do build otimizado e do pacote do inFAMOUS 2; medir de novo com todas as mudanças.
- [x] **Prédios e telhados aparecem em azul forte chapado.** Corrigido em 2026-10-22: o wrap de 28 bits do offset de vértice (veja "Estado atual"); a cidade aparece com as cores certas.
- [ ] **Texto das telas legais sobreposto.** O texto desenhado numa tela legal continua lá na seguinte.
- [ ] As lanes ociosas do job manager são paradas depois de 30 s fazendo polling da mesma linha da fila (`SPU_DMA_REPEAT_SECS`); conferir se elas voltam quando chega trabalho.

### Rumo ao primeiro capítulo

- [x] O vídeo de abertura da fase 1, assistido até o fim (5.584 imagens).
- [x] Tela de carregamento do capítulo 1 ("The Gauntlet", York) e "Press the START button".
- [x] Os primeiros frames de gameplay em York (primeira pessoa, HUD).
- [x] York jogado com controle: inimigos e aliados, tiros, morrer e renascer (2026-10-22).
- [ ] Menu de pausa e opções dentro do jogo.

### Jobs de SPU

- [ ] Um jeito de capturar o código dos jobs do job manager (o próprio PM faz o DMA, então o `SPU_DUMP_MISS` não os vê).
- [ ] Um script que traduz todos os jobs capturados de uma vez.
- [ ] Os jobs de áudio do NextSynth, que rodam no interpretador desde o boot.

### Vídeo e áudio

- [ ] Sincronia de áudio e vídeo num vídeo longo.
- [ ] Conferir o áudio de ouvido num run em janela (`AUDIO_WAV` para gravar).

### Menu

- [x] Tela de título e menu principal.
- [x] Saves: `PROFILE.SAV` e `quick.sav` gravados e lidos em `PS3_HDD0_ROOT`.
- [ ] Opções, `cellOskDialog` (teclado na tela), `cellMsgDialog`.

---

## O que falta para ficar 100% jogável

### Gameplay

- [x] O primeiro capítulo (York) com controle, em janela (2026-10-22; lento e com os erros de renderização acima).
- [ ] O primeiro capítulo (York) com teclado e mouse.
- [ ] Zerar a campanha anotando o que quebra: todos os capítulos, os veículos, os chefes, o tiro alternativo de cada arma.
- [ ] Tela dividida offline (cooperativo e versus): dois controles, duas viewports.

### Gráficos

- [ ] Comparar com o original (RPCS3 ou o console) capítulo por capítulo; abrir um item por bug com captura, como no inFAMOUS 2.

### Desempenho

- [ ] 30 fps (a meta do próprio jogo) no gameplay. ~14 fps medidos antes do build otimizado e do pacote do inFAMOUS 2; novas medições em andamento.
- [x] O cache de vértices entre frames do port do inFAMOUS 2: 5,9 → 14 fps (2026-10-22; `PS3RECOMP_ENG_VCACHE=0` desliga).
- [x] Builds otimizados: runtime -O2 e código PPU traduzido -O1 em toda configuração menos Debug (era -O0) (2026-10-22).
- [ ] Um A/B de fps do pacote de desempenho do inFAMOUS 2 (aplicado, testes passam; veja "Renderização e desempenho" em Feito) e uma medição com todas as mudanças (em andamento).
- [ ] Medir quanto do tempo de SPU é interpretado e cortar (traduzir o que sobrar).

### Estabilidade

- [ ] Sessões longas em janela sem travar; relatório de travamento no estilo do `watch_stall.sh`.

### Controles

- [ ] Teclado e mouse no macOS (o fallback de teclado do `cellPad` do toolkit é só para Windows), mapeamento configurável (`PS3_INPUT_MAP`).
- [x] DualSense/DualShock/Xbox via SDL2 (um DualSense jogou o capítulo 1, 2026-10-22).
- [ ] Vibração (DualShock 3; a atualização v1.7 a adicionou, também na campanha).
- [ ] SIXAXIS: o jogo chama `cellPadSetSensorMode`/`cellPadInfoSensorMode`; descobrir para que usa o movimento e mapear num botão.

### Atualização do jogo (v1.7)

O servidor de atualizações da Sony não lista pacotes para nenhum ID de título do Resistance: o jogo se atualizava pelos próprios servidores de multiplayer (desligados em 2014-04-08). A última versão é a v1.7 (Patch 7), `patch_01`…`patch_07.psarc` em `USRDIR/data`; o `patch_06` traz um `game.self` novo (`/data/new_game.self`, ~8,7 MB), dados de colisão novos para as fases 200–320, menus, fontes, traduções, vibração e suporte a map packs. As notas das atualizações são quase todas de multiplayer. O servidor de revival PSRewired (psrewired.com) serve o Resistance online e, segundo a wiki do RPCS3, a atualização v1.7.

- [ ] Terminar de estabilizar a 01.00 primeiro.
- [ ] Obter a v1.7 deixando o jogo se atualizar pelo PSRewired no RPCS3.
- [ ] Traduzir de novo o `game.self` novo, revalidar o código que depende de endereços, portar o caminho dos dados com patch. Estimativa de 1–3 dias para voltar a ficar jogável.

### Rebuild

- [x] Um script para o rebuild inteiro a partir do disco do próprio jogador (`tools/rebuild.sh`).
- [ ] Os programas das SPUs raw e os jobs de SPU a partir do disco, sem um run de captura.
- [ ] Descriptografia sem RPCS3 (o `decrypt_self.py` do inFAMOUS 2, com o arquivo de chaves do próprio jogador).

---

## Depois de jogável (melhorias)

### Gráficos e resolução

- [ ] Resolução interna acima de 720p, MetalFX/FSR, FOV widescreen, tela cheia/sem borda (reaproveitar o patch do toolkit do inFAMOUS 2: `PS3_WINDOW_MODE`, `PS3_DISPLAY`, `PS3_ANISO`, FXAA, CAS, `PS3_UPSCALE=metalfx`).
- [ ] Destravar acima de 30 fps onde a lógica do jogo permitir.

### App para instalar e jogar (launcher)

- [x] A janela mantém posição, tamanho e tela cheia no exitspawn do menu para a campanha (`PS3RECOMP_WINDOW_STATE`, 2026-10-22).
- [ ] Uma janela única e persistente: um processo launcher dono da janela e um IOSurface compartilhado em que os executáveis do menu e da campanha desenham (em desenvolvimento).
- [ ] O launcher do inFAMOUS 2 (Tauri) adaptado: instalar/rebuild a partir da imagem do disco, configurações, controles, saves, reportar problema, atualizações.
- [ ] Ícone a partir do `ICON0.PNG` do próprio jogador, app bundle com Game Mode no macOS.

### Outras plataformas

- [ ] Linux (Vulkan) e Windows (D3D12), com o launcher e o rebuild rodando nativamente.

### Fora do escopo por enquanto

- [ ] Online (partidas de 40 jogadores, `sceNp`, `sys_net`, chat de voz via `cellMic`): os servidores oficiais acabaram; só com um servidor da comunidade.

---

## Feito e concluído

### Boot e pipeline

- [x] Dump do disco `BCUS98107` v01.00, montado como raiz do VFS.
- [x] `EBOOT.BIN` e `game.self` descriptografados, analisados (EBOOT com 25.022 funções e 183 imports; `game.self` com 18.994 funções e 134 imports) e traduzidos; os dois compilam no macOS/arm64 com Metal.
- [x] Policy module do job manager traduzido na base LS 0xA00 (imagem SPU 1).
- [x] Programas das SPUs raw `1894F66385AFD052` e `5A10969E466A2298` traduzidos na base LS 0x60 (imagens SPU 2 e 3).

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
- [x] O pacote de desempenho e correções do inFAMOUS 2 (testes passam; A/B de fps em andamento): cache de vértices por stream (streams de malhas estáticas guardados à parte do stream de iluminação de cada frame; `PS3RECOMP_ENG_VCACHE_SPLIT=0` desliga), ring de staging do Metal, bind só do estado que mudou (`PS3RECOMP_METAL_BIND_ALL=1` desliga), memo de programas VP/FP e índice de hash de pipelines, decodificação de índices, cache do `mfc_for` da SPU, lock TTAS da lock line, `cflts`/`cfltu` em NEON, espera de fila de eventos da SPU cheia, cor de borda das texturas, lanes de profundidade X32_FLOAT, snapshots de profundidade aposentados por escritas de profundidade, cache de shaders MSL (`PS3_SHADER_CACHE=<pasta>`, `PS3_SHADER_ASYNC=1` opcional), relógio de frame de 1 ms e um present por flip (`PS3_FRAMECLOCK_US=4000` é o ritmo antigo).
- [x] O executável se chama `ResistanceRecomp` (era `MyGameRecomp`), e a janela mantém posição, tamanho e tela cheia no exitspawn.
- [x] Diagnóstico do renascimento: o save do checkpoint bate com o do RPCS3 com diferença de 166 bytes; o bug está no posicionamento no renascimento, não no save (ainda em aberto, veja Bloqueios).

### Ferramentas

- [x] `RES_PEEK` (leitura de memória do guest pelo `port/stubs.cpp`), `MUTEX_WATCH=1|2`, `VDEC_PICLOG`, `PPU_STACKINFO_LOG`, `SPU_RAW_DUMP_DIR`.
- [x] Runs headless com dump de frames e entrada roteirizada (`tools/run.sh`, `start.sh`, `press.sh`, `snap.sh`, `to_campaign.sh`).
- [x] `RSX_ENGINE_PASS_DUMP_TRIGGER=<arquivo>`: o dump por passada começa quando esse arquivo aparece, para um script capturar o frame até onde levou o jogo (2026-10-22).
- [x] Comandos de analógico no `PAD_FILE`, `S lx ly rx ry polls`: segurar os analógicos por script para andar ou virar a câmera (2026-10-22).
- [x] O `RSX_ENGINE_PASS_DUMP_DRAWS` registra o formato e os primeiros bytes dos atributos de vértice de cada draw; `RSX_VPC_LOG=<constante>[,<draw>]` registra uma constante do vertex program a cada 30 frames (`c16` = posição da câmera nos shaders do mundo) (2026-10-22).

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
- [x] Repositório próprio (`resistance-recomp`): fontes do port, um patch do toolkit, `tools/setup_toolkit.sh`, README, créditos e um `.gitignore` que bloqueia dados do jogo, firmware e saves.
