# Roadmap

🌐 **English** · [Português (BR)](ROADMAP.pt-BR.md)

Status of the native Resistance: Fall of Man (`BCUS98107`, disc v01.00,
campaign from the v1.7 update) port to macOS/arm64 and what comes next.
Same layout as the inFAMOUS 2 roadmap.
`[x]` = done and verified; `[ ]` = to do; "(in progress)" = being worked on.

## Status at a glance (2026-10-22)

- **Runs to gameplay:** both executables are lifted, build and run on macOS/arm64 with Metal. Boot, legal screens, first-run install, title screen and main menu lead to a new campaign. `game.self` then plays the level-1 intro movie and loads chapter 1, "The Gauntlet" (York).
- **v1.7 campaign:** the campaign is the game's last update. Its `game.self` (from `patch_06`) is re-lifted and runs from the 01.00 menus: New campaign → York intro, gameplay and respawn verified. The menus stay 01.00.
- **Played:** chapter 1 in a window with a DualSense (SDL2), `tools/play.sh`: first person, HUD, enemies and allies, shooting, dying and respawning on the ground at the checkpoint (the respawn bug is fixed). The town renders with its proper colours (the blue world is fixed).
- **One window:** the `Resistance` launcher owns the window and the game draws into it through shared IOSurfaces; the same window stays up from the menus through the campaign intro (~33 fps).
- **Launcher app:** `launcher/` (Tauri) finds the builds and the disc, rebuilds, keeps the settings and the saves, and plays and stops the game.
- **Shadows and smoke:** the dark rectangles on the ground and the hard-edged smoke are gone in game.
- **Faster:** ~30 fps in gameplay. The inFAMOUS 2 performance stack gives 32.5 fps vs 6.0 fps with its knobs off (A/B, same build, first 60 s of gameplay); 30.5 fps in the town with every fix in, the respawn fix included.
- **Released:** nothing yet.
- **Open:** the menus stay 01.00 (the v1.7 EBOOT exists only as NPDRM); text from earlier legal screens overlaps later ones; keyboard and mouse do not work on macOS (a controller is required); rumble not tested; no Windows or Linux.
- **Next:** rumble with a physical controller; a physical controller with the launcher; a v1.7 fps measurement and a long session; keyboard and mouse on macOS; the legal-screen text overlap.

## Current state (2026-10-22)

The whole pipeline runs from the player's own disc (`tools/rebuild.sh`): the
disc image is mounted and used as the VFS root (`PS3_VFS_ROOT`), both PPU
executables are decrypted, analysed, lifted (EBOOT 25,022 functions;
`game.self` 18,994) and built against ps3recomp v0.17.0 plus this
repository's toolkit patch.

What got it this far:

- **Job manager:** the game's SPURS job-manager policy module (fingerprint
  `4E4076E700C6AFF3`, 11,136 bytes) is lifted at its real local-store base,
  0xA00 (`tools/regen_spu_pm.sh`). Before that it spun for ever on
  `GETLLAR 0x2480`.
- **Metal off the guest thread:** the guest runs on its own thread and the
  macOS main thread turns the run loop (`port/main.cpp`). With the guest on the
  main thread the Metal event pump deadlocked and the FIFO never drained.
- **exitspawn:** `sys_game_process_exitspawn` (`port/stubs.cpp`) execs the
  other build. The parent's data block goes at main-stack-top − 0x1000, where
  `game.self` looks for its `"ELFtastic!"` block, and
  `sys_ppu_thread_get_stack_information` now reports the real main stack
  instead of 0 + 0 (toolkit patch).
- **Movies:** cellVdec decodes through FFmpeg loaded at runtime
  (`-DPS3RECOMP_FFMPEG_RUNTIME=ON`, Homebrew FFmpeg). The movie quad is drawn
  in immediate mode with `DATA2S` positions, which the RSX dispatch did not
  turn into vertices, so the screen stayed black (toolkit patch).
  libavcodec holds back the last B-frame-reordered picture. The player
  waits for it before it ends the movie, so cellVdec drains the decoder once the
  title has taken every picture and sent no AU for 1.5 s (toolkit patch).
- **Raw SPUs:** after the movie, `game.self` streams two programs into raw
  SPUs. Raw SPUs only ever started on Windows. The guest arena also stopped
  below their `0xE0000000` windows, and every raw SPU after the first ran the
  first one's program. All three are fixed (toolkit patch, `port/main.cpp`).
  Both programs are captured with `SPU_RAW_DUMP_DIR` and lifted
  (`tools/regen_rawspu.sh`).
- **The blue world:** the RSX adds `VERTEX_DATA_BASE_OFFSET` to each vertex
  array's offset and wraps the sum at 28 bits (RPCS3's
  `get_vertex_offset_from_base`; Insomniac's engines rely on it).
  Resistance's per-frame world lighting stream (attribute 4, UB256, local
  memory) sits at base + offset = 0x1Fxxxxxx, local 0x0Fxxxxxx. Without the
  wrap the port read past VRAM, got zeros, the lighting weights came out
  (0,0,1) and the world was blue. `rsx_dsp_resolve_vertex_offset` (toolkit
  patch) is used by both vertex fetch paths, with a regression test in
  `test_vertex_fetch_plan.c` that fails without it.
- **Speed:** the cross-frame vertex cache from the inFAMOUS 2 port (5.9 →
  14 fps in gameplay, same build, cache on vs off). The port had been compiled
  at -O0 (no `CMAKE_BUILD_TYPE`, and the toolkit's -O2 only applied to
  Release/RelWithDebInfo); now the runtime is -O2 and the lifted PPU code -O1
  in every configuration but Debug. With the inFAMOUS 2 performance stack:
  32.5 fps in the first 60 s of gameplay vs 6.0 fps with its knobs off
  (`PS3RECOMP_ENG_VCACHE_SPLIT=0`, `PS3RECOMP_METAL_BIND_ALL=1`,
  `PS3_FRAMECLOCK_US=4000`), and 30.5 fps in the town with every fix in.
- **Respawn:** on every checkpoint reload the game writes `SPU_RunCntl`=0 to
  raw SPU 1 (image `5A10969E466A2298`), proxy-DMAs the program back into local
  store and writes `SPU_RunCntl`=1. The runtime ignored the stop while the SPU
  was parked in a channel wait and silently dropped the restart, so the old
  program kept running over the reloaded image and the player was sent up
  into the sky. As in RPCS3's `RawSPUThread.cpp`, `SPU_RunCntl`=0 now really
  stops the raw SPU (the stop request is honoured in channel reads and waits
  and in the out-mailbox stall; `raw_spu_stop` waits for the host thread) and
  `SPU_RunCntl`=1 starts it fresh (`runtime/spu/spu_raw.c`, toolkit patch).
- **One window:** a small launcher, `Resistance`
  (`tools/host_window/host_window.m`), owns the window. `ResistanceRecomp`
  renders on the GPU into IOSurfaces shared with it (the Mach bootstrap name
  is passed in `PS3RECOMP_HOST_SURFACES`) and has no window or Dock icon of
  its own. The window stays up across the EBOOT → `game.self` exitspawn
  (`execv` keeps the pid and the new image reconnects). ~0.35 ms of GPU per
  frame, no CPU copy, no measurable added latency. The host reaps the game
  with a blocking `waitpid`: a non-blocking reap could run a moment before
  the game was reapable, and the host then stayed up with its window open
  after the game had gone (the launcher's Stop hit it). Now a SIGTERM to the
  process group closes both in ~3 s. The title's guest FPS counts each flip
  (it read 0).
- **The v1.7 campaign:** the game's last update is seven archives,
  `patch_01`…`patch_07.psarc`; `patch_06` carries a new `game.self`
  (`/data/new_game.self`; the decrypted ELF's SHA-256 is
  `e1d75e6d4afe…144f`, RPCS3's "game.self Updated", PPU-a08a187d…). It is
  re-lifted (19,706 functions against 19,219 for 01.00) and built as
  `build/BCUS98107_game_v17`; 13 of the 15 WWS job modules and both raw-SPU
  programs changed and are re-lifted (new fingerprints `6D03497950639952` and
  `47C9CB480016D493`; the policy module is byte-identical). New imports:
  `cellSaveDataAutoSave2`/`AutoLoad2`, `_sys_process_atexitspawn`,
  `cellPadSetActDirect` (rumble). At the exitspawn `port/stubs.cpp` runs the
  v1.7 build when it and its ELF exist (`RES_GAME_VERSION=100` keeps 01.00).
  `tools/v17/regen_v17.sh` (`tools/rebuild.sh v17`) extracts `patch_06`,
  decrypts `new_game.self` with `rpcs3 --decrypt`, lifts it, lifts the SPU
  programs (the raw-SPU ones after one capture run) and builds. Verified:
  menus (01.00 EBOOT) → New campaign → the log says
  `campaign = v1.7 game.self` → the York intro, "The Gauntlet"; gameplay and
  respawn work with the `SPU_RunCntl` fix.
- **Menus stay 01.00:** the update's `EBOOT.BIN` is only available in NPDRM
  (digital-edition) form. The disc's 01.00 EBOOT cannot run with the v1.7
  patch data mounted (`patch_04`/`patch_06`: it crashes loading the menu
  fonts; 1/2/3/5/7: it hangs at the exitspawn), so the patch files must not
  be in the play folder; `tools/play.sh` and the launcher refuse to start
  while a `patch_NN.bin` is there. The campaign does not need them: the
  patch data is front-end and multiplayer data, and `game.self` never mounts
  the patch archives. The patch files are the game's official free updates;
  players keep their own (PSRewired, psrewired.com, lets owners have the game
  download its update in RPCS3).
- **Shadows and smoke:** the dark rectangles on the ground and the
  hard-edged smoke are gone in game (checked visually) after the depth fixes
  from the RPCS3 comparison: the shadow comparison runs stored depth OP
  reference, depth textures replicate depth to every lane, and depth
  snapshots are cropped to the region the texture names.

---

## What is missing to reach gameplay

### Blockers

- [x] **SPU jobs time out in gameplay.** Fixed 2026-10-22: the 15 WWS job modules lifted, SPURS kernel-style selection and atomic SPU events (see "Gameplay" under Done). The job manager loads job code into LS (around 0x7C00–0x14000 in its image) and the interpreter runs it at about 8–9 M steps/s. The jobs the first frame needs miss the game's 2 s timeouts. Capture each job's code as it is DMA'd in, lift it (inFAMOUS 2's `lift_spu_jobs.sh` / `relift_jobmod.py` are the model) and register it.
- [x] **Hang after the intro movie (intermittent).** Fixed 2026-10-22: a shared HLE→guest callback stack (see "Gameplay" under Done). A movie-player thread (`func_005F0360 → … → func_005EEFE4`) stays in `sys_mutex_lock`, so the player never reports done and the main thread stays in its load loop (`func_00044DE0`, state 2). `MUTEX_WATCH=2` keeps the lock blocking and reports waiters older than 5 s with the mutex owner; so far the hang has not reproduced under any watch mode. The fair-share mutex policy on macOS did not fix it.
- [x] **The player respawns above the map.** Fixed 2026-10-22: a raw-SPU stop and restart the runtime dropped (see "Current state"). After dying the checkpoint reload sent the player straight up into the sky (draw count ~30–40: sky, far terrain and water); the save was fine. In a test with 12 deaths and respawns the camera always came back to the first life's position (0 of 666 camera samples out of range, 193 of 292 before); a separate run confirmed it (back on the ground at the same spot, then walking normally).
- [x] **Dark rectangles on the ground (shadows) and the smoke.** Fixed 2026-10-22, gone in game (checked visually). The texture shadow comparison was in ARB order; the console and RPCS3 compare stored depth OP reference (RPCS3 issue #19391 shows the same rectangles). Fixed with the other RPCS3-comparison fixes (depth textures replicate depth to every lane, for soft particles and smoke; depth snapshots cropped to the texture's region).
- [x] **Frame rate.** ~30 fps in gameplay (2026-10-22): 32.5 fps with the inFAMOUS 2 stack vs 6.0 fps with its knobs off; 30.5 fps in the town with every fix in.
- [x] **Buildings and roofs render solid bright blue.** Fixed 2026-10-22: the 28-bit vertex offset wrap (see "Current state"); the town renders with its proper colours.
- [ ] **Legal-screen text overlaps.** Text drawn on one legal screen is still there on the next.
- [ ] The job manager's idle lanes are halted after 30 s of polling the same queue line (`SPU_DMA_REPEAT_SECS`); check that halted lanes come back when work arrives.

### Into the first chapter

- [x] The level-1 intro movie, watched to the end (5,584 pictures).
- [x] Chapter 1 loading screen ("The Gauntlet", York) and "Press the START button".
- [x] The first gameplay frames in York (first person, HUD).
- [x] York played with a controller: enemies and allies, shooting, dying and respawning (2026-10-22).
- [ ] Pause menu and in-game options.

### SPU jobs

- [ ] A capture path for the job manager's job code (the PM DMAs it in itself, so `SPU_DUMP_MISS` does not see it).
- [ ] A script that lifts every captured job in one go.
- [ ] The NextSynth audio jobs, which run in the interpreter from boot.

### Video and audio

- [ ] A/V sync over a long movie.
- [ ] Audio check by ear in a windowed run (`AUDIO_WAV` to record).

### Menu

- [x] Title screen and main menu.
- [x] Saves: `PROFILE.SAV` and `quick.sav` written and read under `PS3_HDD0_ROOT`.
- [ ] Options, `cellOskDialog` (on-screen keyboard), `cellMsgDialog`.

---

## What is missing to be 100% playable

### Gameplay

- [x] The first chapter (York) with a controller, in a window (2026-10-22; with the rendering bugs above).
- [ ] The first chapter (York) with keyboard and mouse.
- [ ] Play the whole campaign and write down what breaks: every chapter, the vehicles, the bosses, every weapon's alt-fire.
- [ ] Offline split-screen (co-op and versus): two pads, two viewports.

### Graphics

- [ ] Compare against the original (RPCS3 or the console) chapter by chapter; open an item per bug with a capture, as in inFAMOUS 2.

### Performance

- [x] ~30 fps (the game's own target) in gameplay: 30.5 fps in the town with every fix in (2026-10-22). Keep measuring with every change.
- [x] The cross-frame vertex cache from the inFAMOUS 2 port: 5.9 → 14 fps (2026-10-22; `PS3RECOMP_ENG_VCACHE=0` turns it off).
- [x] Optimised builds: runtime -O2 and lifted PPU code -O1 in every configuration but Debug (it had been -O0) (2026-10-22).
- [x] An fps A/B of the inFAMOUS 2 performance stack: 32.5 fps on vs 6.0 fps with its knobs off (`PS3RECOMP_ENG_VCACHE_SPLIT=0`, `PS3RECOMP_METAL_BIND_ALL=1`, `PS3_FRAMECLOCK_US=4000`), same build, first 60 s of gameplay (2026-10-22).
- [ ] A proper fps measurement of the v1.7 campaign (same A/B as 01.00).
- [ ] Measure how much SPU time is interpreted and cut it (translate what is left).

### Stability

- [ ] Long windowed sessions without hangs (the v1.7 campaign included); a `watch_stall.sh`-style hang report.

### Controls

- [ ] Keyboard and mouse on macOS (the toolkit's keyboard fallback in `cellPad` is Windows-only), configurable mapping (`PS3_INPUT_MAP`).
- [x] DualSense/DualShock/Xbox via SDL2 (a DualSense played chapter 1, 2026-10-22).
- [ ] A physical controller with the launcher: the game process is in the background, so `SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1` is set; not tested with a controller yet.
- [ ] Rumble with a physical controller: the v1.7 `game.self` calls `cellPadSetActDirect`; not tested yet.
- [ ] SIXAXIS: the game calls `cellPadSetSensorMode`/`cellPadInfoSensorMode`; find what it uses motion for and map it to a button.

### Game update (v1.7)

Sony's update server lists no packages for any Resistance title ID: the game patched itself through its multiplayer servers (shut down 2014-04-08). The last version is v1.7 (Patch 7), `patch_01`…`patch_07.psarc` in `USRDIR/data`; `patch_06` carries a new `game.self` (`/data/new_game.self`, ~8.7 MB), new collision data for levels 200–320, menus, fonts, translations, rumble and map-pack support. The patch notes are almost all multiplayer. The PSRewired revival server (psrewired.com) serves Resistance online and, per the RPCS3 wiki, the v1.7 update.

Done 2026-10-22: the campaign runs v1.7 (see "Current state").

- [x] Re-lift the patched `game.self` from `patch_06` (19,706 functions), its 13 changed job modules and both raw-SPU programs; `tools/v17/regen_v17.sh`.
- [x] Run it from the 01.00 menus: the exitspawn picks the v1.7 build; New campaign → York intro, gameplay and respawn verified.
- [x] Keep the patch files out of the play folder (the 01.00 EBOOT cannot run with them): `tools/play.sh` and the launcher refuse to start.
- [ ] v1.7 menus: they need a non-NPDRM v1.7 `EBOOT.BIN`, which does not seem to exist (the update's EBOOT is NPDRM only); likely impossible.
- [ ] Rumble, a long session and an fps measurement on v1.7 (see Controls, Stability, Performance).

### Rebuild

- [x] One script for the whole rebuild from the player's own disc (`tools/rebuild.sh`).
- [ ] The raw-SPU programs and the SPU jobs from the disc without a capture run.
- [ ] Decryption without RPCS3 (inFAMOUS 2's `decrypt_self.py`, with the player's own key file).

---

## After it is playable (improvements)

### Graphics and resolution

- [ ] Internal resolution above 720p, MetalFX/FSR, widescreen FOV, fullscreen/borderless (reuse the inFAMOUS 2 toolkit patch: `PS3_WINDOW_MODE`, `PS3_DISPLAY`, `PS3_ANISO`, FXAA, CAS, `PS3_UPSCALE=metalfx`).
- [ ] Unlock past 30 fps where the game logic allows.

### App to install and play (launcher)

- [x] The window keeps its position, size and fullscreen across the menu → campaign exitspawn (`PS3RECOMP_WINDOW_STATE`, 2026-10-22).
- [x] One persistent window (2026-10-22): the `Resistance` launcher (`tools/host_window/host_window.m`) owns the window and the menu and campaign executables draw into IOSurfaces shared with it; no window or Dock icon of the game's own. Closing the window ends the game; if the launcher dies the game exits within ~2 s; without it the backend opens its own window. Verified from the menus through the campaign intro at ~33 fps.
- [x] The window title's guest FPS counts each flip (it read 0 with the window host) (2026-10-22).
- [x] The launcher app (`launcher/`, Tauri, 2026-10-22): ISO pick and mount, play folder, port-folder detection, campaign version (v1.7/01.00), settings (graphics, performance, game), a saves manager (here and in RPCS3), the game log, debug mode, an updates check, a Desktop and Steam shortcut, English and Brazilian Portuguese. Stop works: the window host now exits when the game does.
- [ ] A physical controller with the launcher (see Controls).
- [ ] Icon from the player's own `ICON0.PNG`, Game Mode app bundle on macOS.

### Other platforms

- [ ] Linux (Vulkan) and Windows (D3D12), with the launcher and the rebuild running natively.

### Out of scope for now

- [ ] Online (40-player matches, `sceNp`, `sys_net`, `cellMic` voice chat): the official servers are gone; only with a community server.

---

## Done and finished

### Boot and pipeline

- [x] Disc dump `BCUS98107` v01.00, mounted as the VFS root.
- [x] `EBOOT.BIN` and `game.self` decrypted, analysed (EBOOT 25,022 functions, 183 imports; `game.self` 18,994 functions, 134 imports) and lifted; both build on macOS/arm64 with Metal.
- [x] Job-manager policy module lifted at LS base 0xA00 (SPU image 1).
- [x] Raw-SPU programs `1894F66385AFD052` and `5A10969E466A2298` lifted at LS base 0x60 (SPU images 2 and 3).

### Front end

- [x] CRT and TLS, `sys_memory`, `cellSysmodule`, `cellDiscGameGetBootDiscInfo`, `cellSysCacheMount`, `cellGameDataCheckCreate2`.
- [x] `cellSpursInitialize`, the job-manager workloads, `cellGcm` init and flips on screen.
- [x] Guest off the macOS main thread.
- [x] Legal screens, the first-run HDD install, title screen, main menu, campaign difficulty and invert-Y prompts, the overwrite warnings.
- [x] `PROFILE.SAV` and `quick.sav` written and read.

### Campaign start

- [x] `sys_game_process_exitspawn` EBOOT → `game.self` with the data block where `game.self` looks for it.
- [x] `sys_ppu_thread_get_stack_information` for the main thread (it used to return 0 + 0, and the first boot read 0xFFFFF000).
- [x] Movies on screen: FFmpeg runtime, immediate-mode `DATA2S` vertices, idle drain of the last reordered picture.
- [x] Raw SPUs on macOS: the POSIX thread, the full 32-bit guest arena, one program per SPU.
- [x] Chapter 1 loads to "Press the START button".

### Gameplay (2026-10-22)

- [x] Hang after the intro movie: every HLE→guest callback ran on one shared guest stack (0xCFFE0000), and the movie player's adec and vdec callbacks overwrote each other's frames; a scoped mutex guard then unlocked NULL and the player mutex stayed locked. Per-host-thread callback stacks (toolkit patch). 3 of 3 runs pass, 5 of 6 hung before.
- [x] The 15 WWS job modules (`C0DEC0DE`) linked into the executables, lifted at their own link base (13 at 0x4000, two audio modules at 0x7000; `tools/regen_jobmods.sh`).
- [x] SPURS kernel-style workload selection (`SPURS_RPCS3_SCHED`, which the port turns on): the policy module's poll gets the workload the kernel would run, readyCount is the PM's, priorities count. Without it the gameplay jobs (`igGeomCullClip`, `SpuBucketerDrawSort`) timed out.
- [x] Atomic SPU event updates: a PPU store's reservation-lost raise overwrote the job manager's list stall-and-notify event, and it waited for its interrupt for ever; the first gameplay frame never finished.
- [x] `sys_mutex_destroy` refuses a held mutex or one with waiters (EBUSY), as lv2 does.
- [x] Chapter 1 played in a window with a DualSense: first person, HUD, enemies and allies, shooting, dying and respawning.
- [x] `tools/play.sh`: windowed play with a controller; saves, install cache and log in `~/Library/Application Support/resistance-recomp/`, frame rate in the window title.
- [x] Host profile of a windowed session: the RSX thread's CPU vertex decode is the bottleneck.
- [x] The job manager's per-poll `[spurs-pm]` log only under `SPURS_PM_TRACE` (it wrote 1.9 million lines in a short session, enough to cost frame rate).

### Rendering and performance (2026-10-22)

- [x] The blue world: vertex array offsets plus `VERTEX_DATA_BASE_OFFSET` wrap at 28 bits (`rsx_dsp_resolve_vertex_offset`, both fetch paths, regression test). The town renders with its proper colours.
- [x] Cross-frame vertex cache from the inFAMOUS 2 port: 5.9 → 14 fps in gameplay (same -O0 build, cache on vs off).
- [x] Optimised builds: the port had been compiled at -O0; now the runtime is -O2 and the lifted PPU code -O1 for every configuration but Debug.
- [x] Fixes from a comparison with RPCS3 (all video tests pass; in-game effect being evaluated): shadow comparison direction (stored depth OP reference), depth snapshots cropped to the texture size and released (they leaked every frame), depth textures replicate depth to every lane, compressed textures never resolve from render targets, PPU threads start with VSCR[NJ]=1, no `getenv` on every SPU MFC channel write.
- [x] The inFAMOUS 2 performance and fix stack (tests pass; fps A/B: 32.5 vs 6.0 fps): per-stream vertex cache (static mesh streams cached apart from the per-frame lighting stream; `PS3RECOMP_ENG_VCACHE_SPLIT=0` turns it off), Metal staging ring, binding only changed state (`PS3RECOMP_METAL_BIND_ALL=1` turns it off), VP/FP program memo and pipeline hash index, index decode, SPU `mfc_for` cache, lock-line TTAS lock, NEON `cflts`/`cfltu`, SPU event queue full wait, texture border colour, X32_FLOAT depth lanes, depth snapshots retired by depth writes, MSL shader cache (`PS3_SHADER_CACHE=<dir>`, `PS3_SHADER_ASYNC=1` opt-in), a 1 ms frame clock and one present per flip (`PS3_FRAMECLOCK_US=4000` is the old pacing).
- [x] The executable is named `ResistanceRecomp` (it was `MyGameRecomp`), and the window keeps its position, size and fullscreen across exitspawn.
- [x] Respawn diagnosis: the checkpoint save matches RPCS3's to within 166 bytes; the bug was not in the save.
- [x] Respawn fixed: `SPU_RunCntl`=0 really stops a raw SPU and `SPU_RunCntl`=1 starts it fresh (modelled on RPCS3's `RawSPUThread.cpp`); 12 deaths in a test, the camera back at the first life's position every time.
- [x] One persistent window through the `Resistance` launcher and shared IOSurfaces (~0.35 ms GPU per frame, no CPU copy); the window survives the exitspawn.

### The v1.7 campaign and the launcher (2026-10-22)

- [x] The v1.7 `game.self` (`patch_06` `/data/new_game.self`) re-lifted and built as `build/BCUS98107_game_v17`; 13 of the 15 job modules and both raw-SPU programs re-lifted (`6D03497950639952`, `47C9CB480016D493`); the policy module unchanged.
- [x] The exitspawn runs the v1.7 build when it and its ELF exist (`RES_GAME_VERSION=100` keeps 01.00); menus → New campaign → York intro, gameplay and respawn verified.
- [x] `tools/v17/regen_v17.sh` (`tools/rebuild.sh v17`), `tools/v17/psarc_extract.py`; `tools/play.sh` says which campaign runs and refuses to start with the update's `patch_NN.bin` in the play folder.
- [x] Shadows and smoke fixed in game (checked visually).
- [x] The launcher app (`launcher/`), with a working Stop (the window host reaps the game reliably).
- [x] The title's guest FPS counts each flip.

### Tools

- [x] `RES_PEEK` (guest memory peeks from `port/stubs.cpp`), `MUTEX_WATCH=1|2`, `VDEC_PICLOG`, `PPU_STACKINFO_LOG`, `SPU_RAW_DUMP_DIR`.
- [x] Headless runs with frame dumps and scripted input (`tools/run.sh`, `start.sh`, `press.sh`, `snap.sh`, `to_campaign.sh`).
- [x] `RSX_ENGINE_PASS_DUMP_TRIGGER=<file>`: the per-pass dump starts when that file appears, so a script captures the frame it has steered the game to (2026-10-22).
- [x] `PAD_FILE` stick commands, `S lx ly rx ry polls`: hold the sticks from a script to walk or turn the camera (2026-10-22).
- [x] `RSX_ENGINE_PASS_DUMP_DRAWS` logs each draw's vertex attribute formats and first bytes; `RSX_VPC_LOG=<constant>[,<draw>]` logs a vertex-program constant every 30 frames (`c16` = camera position in the world shaders) (2026-10-22).

---

## Giving back

Pull requests to [sp00nznet/ps3recomp](https://github.com/sp00nznet/ps3recomp), from branches on [erikomis/ps3recomp](https://github.com/erikomis/ps3recomp), each with a test:

- [ ] [#463](https://github.com/sp00nznet/ps3recomp/pull/463) RSX dispatch: every ATTR0 immediate encoding emits a vertex (the black movies).
- [ ] [#469](https://github.com/sp00nznet/ps3recomp/pull/469) `sys_ppu_thread_get_stack_information` for the main thread.
- [ ] [#471](https://github.com/sp00nznet/ps3recomp/pull/471) cellVdec: idle drain of the pictures libavcodec holds back.
- [ ] [#473](https://github.com/sp00nznet/ps3recomp/pull/473) Raw SPUs on POSIX, one program per SPU.
- [ ] [#474](https://github.com/sp00nznet/ps3recomp/pull/474) `MUTEX_WATCH` diagnostic.
- [ ] [#475](https://github.com/sp00nznet/ps3recomp/pull/475) Template: the POSIX guest arena spans the raw-SPU windows.
- [ ] [#477](https://github.com/sp00nznet/ps3recomp/pull/477) Template: the guest off the macOS main thread.
- [ ] [#488](https://github.com/sp00nznet/ps3recomp/pull/488) lift_jobmods: each job module lifted at the link base in its header.
- [ ] [#489](https://github.com/sp00nznet/ps3recomp/pull/489) HLE→guest callbacks on a stack per host thread (the post-movie hang).
- [ ] [#490](https://github.com/sp00nznet/ps3recomp/pull/490) `sys_mutex_destroy` returns EBUSY for a held mutex or one with waiters.
- [ ] [#491](https://github.com/sp00nznet/ps3recomp/pull/491) `SPU_IRQWAIT_DIAG` diagnostic.
- [ ] [#492](https://github.com/sp00nznet/ps3recomp/pull/492) Atomic SPU event updates (the first gameplay frame).
- [x] [#493](https://github.com/sp00nznet/ps3recomp/pull/493) SPURS kernel-style selection: closed; it stays in this repository's toolkit patch.
- [ ] [#504](https://github.com/sp00nznet/ps3recomp/pull/504) Vertex array addresses wrap at 28 bits (the blue world).
- [ ] [#505](https://github.com/sp00nznet/ps3recomp/pull/505) `SPU_RunCntl`=0 stops a running raw SPU (the respawn; stacked on #473).
- [ ] [#506](https://github.com/sp00nznet/ps3recomp/pull/506) Optimise a build configured without `CMAKE_BUILD_TYPE`.
- [ ] [#507](https://github.com/sp00nznet/ps3recomp/pull/507) A depth snapshot is the region the texture names (stacked on #502).
- [ ] [#508](https://github.com/sp00nznet/ps3recomp/pull/508) A compressed texture is never a render target's output.
- [ ] [#509](https://github.com/sp00nznet/ps3recomp/pull/509) macOS: one window across an exitspawn (`tools/host_window`, Metal host-surface mode).
- [x] Its own repository (`resistance-recomp`): port sources, a toolkit patch, `tools/setup_toolkit.sh`, README, credits, and a `.gitignore` that blocks game data, firmware and saves.
