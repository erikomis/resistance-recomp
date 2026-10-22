# Roadmap

🌐 [English](ROADMAP.md) · **Português (BR)**

Estado do port nativo de Resistance: Fall of Man (`BCUS98107`, disco, v01.00)
para macOS/arm64 e o que vem a seguir. Mesmo formato do roadmap do inFAMOUS 2.
`[x]` = feito e verificado; `[ ]` = a fazer; "(em andamento)" = sendo feito.

## Resumo (2026-10-21)

- **Roda:** os dois executáveis (`EBOOT.BIN` e `game.self`) estão traduzidos, compilam e rodam no macOS/arm64 com Metal. O front end vai do boot pelas telas legais, pela instalação no HDD da primeira execução, pela tela de título e pelo menu principal (Campanha / Cooperativo / Multiplayer / Opções) até as perguntas de dificuldade da campanha e de inverter o eixo Y; os saves (`PROFILE.SAV`, `quick.sav`) são gravados e lidos de volta. Começar uma campanha faz exitspawn do `game.self`, que dá boot direto no vídeo de abertura da fase 1.
- **Travado em:** os vídeos do jogo aparecem pretos (a imagem é desenhada como um quad em modo imediato; em investigação), então ainda não dá para assistir à abertura e o gameplay ainda não foi alcançado.
- **Lançado:** nada ainda.
- **Em andamento:**
  - [ ] (em andamento) Vídeos pretos: o cellVdec decodifica as imagens (FFmpeg em runtime), mas o quad em modo imediato que deveria mostrá-las não põe nada na tela.
  - [ ] (em andamento) Texto das telas legais anteriores fica na tela e se sobrepõe às seguintes.
- **Próximo:** o vídeo de abertura na tela, depois o primeiro capítulo (York) em janela.

## Estado atual (2026-10-21)

O pipeline inteiro roda a partir do disco do próprio jogador
(`tools/rebuild.sh`): a imagem do disco é montada e usada como raiz do VFS
(`PS3_VFS_ROOT`), os dois executáveis PPU são descriptografados
(`game/BCUS98107/EBOOT.elf`, `game/BCUS98107/modules/game.elf`), analisados,
traduzidos (EBOOT com 25.022 funções; `game.self` com 18.994) e compilados
contra o ps3recomp v0.17.0 mais o patch de toolkit deste repositório.

O que pôs o front end na tela:

- **Job manager:** o policy module do job manager SPURS do jogo (fingerprint
  `4E4076E700C6AFF3`, 11.136 bytes) é traduzido na base real de local store,
  0xA00 (`tools/regen_spu_pm.sh`). Antes disso ele ficava preso para sempre em
  `GETLLAR 0x2480` e a thread principal esperava por ele em `usleep(30)`.
- **Metal fora da thread do guest:** o guest roda na própria thread e a thread
  principal do macOS gira o run loop (`port/main.cpp`). Com o guest na thread
  principal, o pump de eventos do backend Metal travava e o FIFO nunca
  esvaziava.
- **Pilha da thread principal:** `sys_ppu_thread_get_stack_information`
  devolvia 0 + 0 para a thread principal; agora devolve a pilha real (patch do
  toolkit).
- **exitspawn:** `sys_game_process_exitspawn` está implementado em
  `port/stubs.cpp` como um `execv` do outro build, com o bloco de dados do pai
  em topo-da-pilha-principal − 0x1000, onde o `game.self` procura o bloco
  `"ELFtastic!"`.
- **Vídeo:** o cellVdec decodifica pelo FFmpeg carregado em runtime
  (`-DPS3RECOMP_FFMPEG_RUNTIME=ON`, FFmpeg do Homebrew).

Os jobs de SPU que o jogo carrega em tempo de execução (os jobs de áudio
NextSynth e outros) ainda rodam no interpretador de SPU do toolkit.

---

## O que falta para chegar ao gameplay

### Bloqueios

- [ ] **Vídeos pretos.** A abertura da fase 1 (e os vídeos do front end) decodificam, mas o quad que apresenta a imagem é desenhado em modo imediato e nada chega ao frame. Descobrir o que o draw precisa (dados de vértice pelo caminho de modo imediato, a textura para onde a imagem é enviada, o formato dela) e comparar com o RPCS3.
- [ ] **Texto das telas legais sobreposto.** O texto desenhado numa tela legal continua lá na próxima: achar o que limpa (ou deveria limpar) o texto anterior.
- [ ] `[vm] OOB access 0xFFFFF000 (+1)` logo depois das tentativas de abrir `parameters.txt`: conferir se sumiu com a correção da informação de pilha e, se não, achar o PC do guest (`PS3_NULL_READ_STACK=1`).

### Até o primeiro capítulo

- [ ] Passar (ou pular) o vídeo de abertura e entrar em York.
- [ ] Telas de carregamento e a primeira fase carregando do `game.psarc` (6,26 GB).
- [ ] Menu de pausa e opções dentro do jogo.

### Jobs de SPU

- [ ] Jobs de SPU que o jogo carrega em tempo de execução: capturar cada fingerprint nova (`SPU_DUMP_MISS=<dir>`, como foi com `spujob_4E4076E700C6AFF3_11136`), traduzir, registrar. Começar pelos jobs de áudio NextSynth (`ps3soundstream`).
- [ ] Um script que traduz todos os jobs capturados de uma vez (`lift_spu_jobs.sh`, `relift_jobmod.py`, `lift_relocmods.py` do inFAMOUS 2 são o modelo).

### Vídeo e áudio

- [ ] Tempo dos vídeos e sincronia de áudio e vídeo quando a imagem aparecer.
- [ ] Áudio: portas do `cellAudio` e a event queue de notificação; os jobs de SPU de sound stream acima. Gravar com `AUDIO_WAV` e ouvir.

### Menu

- [x] Tela de título e menu principal.
- [ ] Opções, `cellOskDialog` (teclado na tela), `cellMsgDialog`.
- [x] Saves: `PROFILE.SAV` e `quick.sav` gravados e lidos em `PS3_HDD0_ROOT`.
- [ ] Novo jogo até a primeira cutscene, assistida até o fim.

---

## O que falta para ficar 100% jogável

### Gameplay

- [ ] O primeiro capítulo (York) com teclado e com controle, em janela.
- [ ] Zerar a campanha anotando o que quebra: todos os capítulos, os veículos, os chefes, o tiro alternativo de cada arma.
- [ ] Tela dividida offline (cooperativo e versus): dois controles, duas viewports.

### Gráficos

- [ ] Comparar com o original (RPCS3 ou o console) capítulo por capítulo; abrir um item por bug com captura, como no inFAMOUS 2.

### Desempenho

- [ ] 30 fps (a meta do próprio jogo) no gameplay. Perfil por thread (`PS3_FTRACE`) assim que houver gameplay.
- [ ] Medir quanto do tempo de SPU é interpretado e cortar (traduzir o que sobrar).

### Estabilidade

- [ ] Sessões longas em janela sem travar; relatório de travamento no estilo do `watch_stall.sh`.

### Controles

- [ ] Teclado e mouse (o `cellPad` do toolkit), mapeamento configurável (`PS3_INPUT_MAP`).
- [ ] DualSense/DualShock/Xbox via SDL2, vibração.
- [ ] SIXAXIS: o jogo chama `cellPadSetSensorMode`/`cellPadInfoSensorMode`; descobrir para que usa o movimento e mapear num botão.

### Rebuild

- [x] Um script para o rebuild inteiro a partir do disco do próprio jogador (`tools/rebuild.sh`: descriptografa com o RPCS3, analisa, traduz PPU e SPU, compila). A análise e a tradução que ele gera são idênticas, byte a byte, às testadas.
- [ ] Descriptografar sem o RPCS3 (o `decrypt_self.py` do inFAMOUS 2, com o arquivo de chaves do próprio jogador).

---

## Depois de jogável (melhorias)

### Gráficos e resolução

- [ ] Resolução interna acima de 720p, MetalFX/FSR, FOV widescreen, tela cheia/sem borda (reaproveitar o patch do toolkit do inFAMOUS 2: `PS3_WINDOW_MODE`, `PS3_DISPLAY`, `PS3_ANISO`, FXAA, CAS, `PS3_UPSCALE=metalfx`).
- [ ] Destravar acima de 30 fps onde a lógica do jogo permitir.

### App para instalar e jogar (launcher)

- [ ] O launcher do inFAMOUS 2 (Tauri) adaptado: instalar/rebuild a partir da imagem do disco, configurações, controles, saves, reportar problema, atualizações.
- [ ] Ícone a partir do `ICON0.PNG` do próprio jogador, app bundle com Game Mode no macOS.

### Outras plataformas

- [ ] Linux (Vulkan) e Windows (D3D12), com o launcher e o rebuild rodando nativamente.

### Fora do escopo por enquanto

- [ ] Online (partidas de 40 jogadores, `sceNp`, `sys_net`, chat de voz via `cellMic`): os servidores oficiais acabaram; só com um servidor da comunidade.

---

## Feito e concluído

### Boot e pipeline

- [x] Dump do disco `BCUS98107` v01.00 (`VERSION` 01.00 no PARAM.SFO), montado como raiz do VFS.
- [x] `EBOOT.BIN` e `game.self` descriptografados para ELF.
- [x] Análise: `EBOOT` com 25.022 funções, 183 imports em 14 bibliotecas; `game.self` com 18.994 funções, 134 imports em 11 bibliotecas.
- [x] PPU traduzido para C++ nos dois, e os dois projetos compilam no macOS/arm64 com Metal.
- [x] Policy module do job manager (`4E4076E700C6AFF3`, 11.136 bytes; guest 0x10191A80 no EBOOT, 0x10176E80 no `game.self`) traduzido na base de LS 0xA00 e registrado como imagem SPU 1 (`port/spu/spu_workloads.c`).

### Front end

- [x] CRT e TLS, `sys_memory`, carregamento de módulos pelo `cellSysmodule`, `cellDiscGameGetBootDiscInfo`, `cellSysCacheMount`, `cellGameDataCheckCreate2`.
- [x] `cellSpursInitialize` (3 SPUs), os workloads do job manager rodando, inicialização do `cellGcm` e flips na tela.
- [x] Guest fora da thread principal do macOS (fim do travamento do pump de eventos do Metal).
- [x] Telas legais, a instalação no HDD da primeira execução ("initial setup", os arquivos de cache `*.done`), tela de título, menu principal.
- [x] Perguntas de dificuldade da campanha e de inverter o eixo Y; `PROFILE.SAV` e `quick.sav` gravados e lidos.
- [x] `sys_game_process_exitspawn` EBOOT → `game.self` (um execv do outro build; o bloco de dados em topo-da-pilha-principal − 0x1000), e o `game.self` dando boot no vídeo de abertura da fase 1.
- [x] `sys_ppu_thread_get_stack_information` para a thread principal.
- [x] cellVdec pelo FFmpeg em runtime.

---

## Retribuindo

- [ ] As correções do toolkit feitas neste port como PRs para o [ps3recomp](https://github.com/sp00nznet/ps3recomp), cada uma com teste quando possível (primeiro a informação de pilha da thread principal).
- [x] Repositório próprio (`resistance-recomp`), como o `infamous2-recomp`: fontes do port, um patch do toolkit, `tools/setup_toolkit.sh`, README, créditos e um `.gitignore` que bloqueia dados do jogo, firmware e saves.
