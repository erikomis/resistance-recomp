# Roadmap

🌐 **English** · [Português (BR)](ROADMAP.pt-BR.md)

Status of the native Resistance: Fall of Man (`BCUS98107`, disc, v01.00) port
to macOS/arm64 and what comes next. Same layout as the inFAMOUS 2 roadmap.
`[x]` = done and verified; `[ ]` = to do; "(in progress)" = being worked on.

## Status at a glance (2026-10-22)

- **Runs to gameplay:** both executables are lifted, build and run on macOS/arm64 with Metal. Boot, legal screens, first-run install, title screen and main menu lead to a new campaign. `game.self` then plays the level-1 intro movie and loads chapter 1, "The Gauntlet" (York).
- **Played:** chapter 1 in a window with a DualSense (SDL2), `tools/play.sh`: first person, HUD, enemies and allies, shooting, dying and respawning. The town renders with its proper colours (the blue world is fixed).
- **Faster:** the cross-frame vertex cache took gameplay from 5.9 to ~14 fps (same unoptimised build). Since then the build is optimised (runtime -O2, lifted code -O1) and the inFAMOUS 2 performance stack is in; new measurements in progress.
- **Released:** nothing yet.
- **Open:** after dying, the player respawns far above the map; dark rectangles on the ground (shadows) and the smoke, to check in game after the RPCS3-comparison fixes; text from earlier legal screens overlaps later ones; keyboard and mouse do not work on macOS (a controller is required); no rumble; no Windows or Linux.
- **Next:** the respawn position (top priority), the in-game check of shadows and smoke, an fps measurement with every change, then the single-window launcher and the v1.7 update.

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
  in every configuration but Debug.

---

## What is missing to reach gameplay

### Blockers

- [x] **SPU jobs time out in gameplay.** Fixed 2026-10-22: the 15 WWS job modules lifted, SPURS kernel-style selection and atomic SPU events (see "Gameplay" under Done). The job manager loads job code into LS (around 0x7C00–0x14000 in its image) and the interpreter runs it at about 8–9 M steps/s. The jobs the first frame needs miss the game's 2 s timeouts. Capture each job's code as it is DMA'd in, lift it (inFAMOUS 2's `lift_spu_jobs.sh` / `relift_jobmod.py` are the model) and register it.
- [x] **Hang after the intro movie (intermittent).** Fixed 2026-10-22: a shared HLE→guest callback stack (see "Gameplay" under Done). A movie-player thread (`func_005F0360 → … → func_005EEFE4`) stays in `sys_mutex_lock`, so the player never reports done and the main thread stays in its load loop (`func_00044DE0`, state 2). `MUTEX_WATCH=2` keeps the lock blocking and reports waiters older than 5 s with the mutex owner; so far the hang has not reproduced under any watch mode. The fair-share mutex policy on macOS did not fix it.
- [ ] **The player respawns above the map** (top priority, in progress). After dying and respawning from the checkpoint the player comes back far above the map: the draw count drops from ~250–500 to ~30–40 (only sky, far terrain and water). The checkpoint save (`USRDIR/quick.sav`, 52,832 bytes) matches RPCS3's to within 166 bytes, so the save is fine and the respawn placement is wrong. Hypotheses: PPU/SPU physics on reload; frame-time (dt) sensitivity after a long load (RPCS3's own 60 fps patch needs a minimum timestep). `RSX_VPC_LOG=16` follows the camera position.
- [ ] **Dark rectangles on the ground (shadows) and the smoke** (in progress). The texture shadow comparison was in ARB order; the console and RPCS3 compare stored depth OP reference (RPCS3 issue #19391 shows the same rectangles). Fixed with the other RPCS3-comparison fixes (depth textures replicate depth to every lane, for soft particles and smoke); all video tests pass, the in-game check is pending.
- [ ] **Frame rate** (in progress). ~14 fps in gameplay with the vertex cache, measured before the optimised build and the inFAMOUS 2 stack; measure again with every change.
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

- [x] The first chapter (York) with a controller, in a window (2026-10-22; slow and with the rendering bugs above).
- [ ] The first chapter (York) with keyboard and mouse.
- [ ] Play the whole campaign and write down what breaks: every chapter, the vehicles, the bosses, every weapon's alt-fire.
- [ ] Offline split-screen (co-op and versus): two pads, two viewports.

### Graphics

- [ ] Compare against the original (RPCS3 or the console) chapter by chapter; open an item per bug with a capture, as in inFAMOUS 2.

### Performance

- [ ] 30 fps (the game's own target) in gameplay. ~14 fps measured before the optimised build and the inFAMOUS 2 stack; new measurements in progress.
- [x] The cross-frame vertex cache from the inFAMOUS 2 port: 5.9 → 14 fps (2026-10-22; `PS3RECOMP_ENG_VCACHE=0` turns it off).
- [x] Optimised builds: runtime -O2 and lifted PPU code -O1 in every configuration but Debug (it had been -O0) (2026-10-22).
- [ ] An fps A/B of the inFAMOUS 2 performance stack (applied, tests pass; see "Rendering and performance" under Done) and a measurement with every change (in progress).
- [ ] Measure how much SPU time is interpreted and cut it (translate what is left).

### Stability

- [ ] Long windowed sessions without hangs; a `watch_stall.sh`-style hang report.

### Controls

- [ ] Keyboard and mouse on macOS (the toolkit's keyboard fallback in `cellPad` is Windows-only), configurable mapping (`PS3_INPUT_MAP`).
- [x] DualSense/DualShock/Xbox via SDL2 (a DualSense played chapter 1, 2026-10-22).
- [ ] Rumble (DualShock 3; the v1.7 update added it, also in the campaign).
- [ ] SIXAXIS: the game calls `cellPadSetSensorMode`/`cellPadInfoSensorMode`; find what it uses motion for and map it to a button.

### Game update (v1.7)

Sony's update server lists no packages for any Resistance title ID: the game patched itself through its multiplayer servers (shut down 2014-04-08). The last version is v1.7 (Patch 7), `patch_01`…`patch_07.psarc` in `USRDIR/data`; `patch_06` carries a new `game.self` (`/data/new_game.self`, ~8.7 MB), new collision data for levels 200–320, menus, fonts, translations, rumble and map-pack support. The patch notes are almost all multiplayer. The PSRewired revival server (psrewired.com) serves Resistance online and, per the RPCS3 wiki, the v1.7 update.

- [ ] Finish stabilising 01.00 first.
- [ ] Get v1.7 by letting the game update itself through PSRewired in RPCS3.
- [ ] Re-lift the new `game.self`, revalidate the address-specific code, port the patched data path. Estimated 1–3 days to get back to playable.

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
- [ ] One persistent window: a launcher process that owns the window and a shared IOSurface the menu and campaign executables draw into (in development).
- [ ] The inFAMOUS 2 launcher (Tauri) adapted: install/rebuild from the disc image, settings, controls, saves, report a problem, updates.
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
- [x] The inFAMOUS 2 performance and fix stack (tests pass; fps A/B in progress): per-stream vertex cache (static mesh streams cached apart from the per-frame lighting stream; `PS3RECOMP_ENG_VCACHE_SPLIT=0` turns it off), Metal staging ring, binding only changed state (`PS3RECOMP_METAL_BIND_ALL=1` turns it off), VP/FP program memo and pipeline hash index, index decode, SPU `mfc_for` cache, lock-line TTAS lock, NEON `cflts`/`cfltu`, SPU event queue full wait, texture border colour, X32_FLOAT depth lanes, depth snapshots retired by depth writes, MSL shader cache (`PS3_SHADER_CACHE=<dir>`, `PS3_SHADER_ASYNC=1` opt-in), a 1 ms frame clock and one present per flip (`PS3_FRAMECLOCK_US=4000` is the old pacing).
- [x] The executable is named `ResistanceRecomp` (it was `MyGameRecomp`), and the window keeps its position, size and fullscreen across exitspawn.
- [x] Respawn diagnosis: the checkpoint save matches RPCS3's to within 166 bytes; the bug is in the respawn placement, not the save (still open, see Blockers).

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
- [x] Its own repository (`resistance-recomp`): port sources, a toolkit patch, `tools/setup_toolkit.sh`, README, credits, and a `.gitignore` that blocks game data, firmware and saves.
