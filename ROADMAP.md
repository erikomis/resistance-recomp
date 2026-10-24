# Roadmap

🌐 **English** · [Português (BR)](ROADMAP.pt-BR.md)

Status of the native Resistance: Fall of Man (`BCUS98107`, disc v01.00,
campaign from the v1.7 update) port to macOS/arm64 and what comes next.
Same layout as the inFAMOUS 2 roadmap.
`[x]` = done and verified; `[ ]` = to do; "(in progress)" = being worked on.

## Status at a glance (2026-10-24)

- **Runs to gameplay:** both executables are lifted, build and run on macOS/arm64 with Metal. Boot, legal screens, first-run install, title screen and main menu lead to a new campaign. `game.self` then plays the level-1 intro movie and loads chapter 1, "The Gauntlet" (York).
- **v1.7 campaign:** the campaign is the game's last update. Its `game.self` (from `patch_06`) is re-lifted and runs from the 01.00 menus: New campaign → York intro, gameplay and respawn verified. The menus stay 01.00.
- **Played:** chapter 1 in a window with a DualSense (SDL2), `tools/play.sh`: first person, HUD, enemies and allies, shooting, dying and respawning on the ground at the checkpoint (the respawn bug is fixed). The town renders with its proper colours (the blue world is fixed).
- **One window:** the `Resistance` launcher owns the window and the game draws into it through shared IOSurfaces; the same window stays up from the menus through the campaign intro (~33 fps).
- **Launcher app:** `launcher/` (Tauri) finds the builds and the disc, rebuilds, keeps the settings and the saves, and plays and stops the game.
- **Shadows and smoke:** the dark rectangles on the ground and the hard-edged smoke are gone in game.
- **At the game's own cap:** the campaign reaches the game's ~30–33 fps cap (Resistance runs at 30 fps on the PS3); the PPU's remaining "busy" time is largely the game's frame limiter. The GPU depth-as-RGBA8 resolve took the RSX thread from 54% to 12% busy (2026-10-23).
- **New, needs a play test:** keyboard and mouse on macOS with `input.toml` remapping, rumble, window modes/vsync/MetalFX, SMAA and filters, texture packs and the internal resolution (experimental), ported from the inFAMOUS 2 work; tests pass. The launcher has Controls and Graphics pages for them.
- **60 fps and field of view (2026-10-23):** the community's RPCS3 patches carried over (`port/patches.cpp`, `port/patches/*.insn.json`), all opt-in: `RES_UNLOCK_FPS=1` in the campaign (01.00 and v1.7) at the normal game speed, `RES_MENU_60FPS=1` menus 32 → 59 fps, `RES_FOV`; the launcher's "60 fps" switch sets both.
- **60 fps pacing (2026-10-24):** with `RES_UNLOCK_FPS=1` the port now runs the vblank clock at 120 Hz (`PS3_VBLANK_HZ`, still settable): on 16 ms ticks a 17 ms frame waited for the tick after next. Measured in gameplay: 88.5 fps (it was ~50–56), game speed unchanged. `PS3_FLIP_MAX_HZ` (off by default) caps the flips; capping at 60 measured worse (50 fps).
- **Shadows on the ground (2026-10-24):** dark hard-edged rectangles on the ground: the draw engine's surface and depth tables (64 slots each) filled up and nothing was ever evicted. Resistance packs its shadow maps into a per-frame pool whose addresses shift, so once the tables were full new shadow passes were silently dropped and the receivers sampled empty maps. The tables now evict the least recently used entry; the evictions show in a play session's log, a play test of the scene is still to do.
- **Cooperative split-screen (2026-10-24):** player 1's (left) view was black: a surface clear ignored the scissor, so player 2's clear wiped player 1's half. Clears are now scissored (Metal partial-clear passes); both halves render in game.
- **Polygon offset and invariant positions (2026-10-24):** the polygon offset is honoured (it was ignored; the game uses it on the character shadow maps, terrain decals and projected shadows), and the vertex position is compiled invariant (the foliage's depth pre-pass and its EQUAL colour pass). Real ZPASS occlusion counts are implemented but off by default (`PS3RECOMP_ENG_ZPASS=1`): they cost frame rate at 60 fps and showed no visible gain.
- **Robustness (2026-10-24):** over two hours of scripted play with no crash: every EBOOT menu and extra, offline split-screen multiplayer, pause and options, Save & Quit → Continue (exitspawn both ways), death and respawn at 30 and 60 fps, a fresh empty HDD, a corrupted `quick.sav`, a 25-minute 60 fps soak and a 22-minute idle one. Multiplayer → Online Game no longer hangs offline (the network-start dialog answers "cancelled"; NP tickets added), the quit-time NID is `cellSysconfAbort` (added with `cellSysconfOpen`), and `cellPadInfoSensorMode`/`cellPadSetSensorMode` are in: a full run has no unresolved NIDs.
- **Legal screens fixed (2026-10-23):** boot goes logos → title "Press the START button" with no overlapping text; the runtime had told the game it was on a Japanese console (licence area).
- **Audio (2026-10-23):** the macOS mixer keeps real time (it alternated 2.4× / 0.95×); volume, mute and output device (`PS3_AUDIO_*`, also in the launcher).
- **Continue fixed (2026-10-23):** Continue from a checkpoint froze on loading (v1.7 and 01.00): the second raw-SPU program carries a parameter area that changes with the level and checkpoint, so it came in under a fingerprint no build had registered. It is now matched with that area masked, and both raw-SPU programs are cut out of the ELF (no capture run). Continue from a saved checkpoint, new campaign and respawn verified on both campaigns.
- **Movie sound (2026-10-23):** every movie was silent (cellAdec decoded nothing); the AC3 5.1 tracks are now decoded through FFmpeg and the POSIX mixer paces itself on the clock. The intro movie plays at −25…−34 dBFS on all six channels, 0–0.2% silent blocks.
- **Fog, smoke and the fake water (2026-10-23):** the river level showed a vertically streaked "reflection": the fragment shaders' `WPOS` ignored the window's bottom-left origin, so the ground-fog and smoke shaders read the depth buffer upside down. The level now shows the dry creek bed with soft fog (both campaigns). Distant soldiers no longer stand on dark rectangles (swizzled 16-bit depth shadow maps written back to guest memory).
- **Toolkit:** 18 fixes from upstream ps3recomp pull requests and the inFAMOUS 2 work (see CREDITS); a regression run with all of them renders with no errors, respawn OK.
- **Released:** v0.1.0 and v0.2.0 (2026-10-23) and v0.2.1 (2026-10-24), pre-releases: the macOS launcher and a Windows preview.
- **Open:** dark speckles on the foliage (it looks like the game's own content, grainy leaf alpha; not yet compared with RPCS3); an eye glitch on a character that has not been reproduced (it needs a repro); the menu music sometimes has extra centre/LFE content (+4–6 dB, from the game's synth, varies per run); the menus stay 01.00 (the v1.7 EBOOT exists only as NPDRM); keyboard, mouse, rumble and the new graphics settings not yet played with; a physical controller with the launcher; Windows untested (the game, and the new audio mixer), no Linux.
- **Next:** a play test of the shadow-pool fix in the scene where the rectangles showed; a repro of the eye glitch; compare the foliage with RPCS3; a play session with keyboard and mouse, rumble and the new graphics settings; a physical controller with the launcher; Windows.

## Current state (2026-10-24)

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
  Both programs are linked into the ELF; `tools/extract_rawspu_elf.py` cuts
  them out by the `0xDEADDEAD` header the loader uses and
  `tools/regen_rawspu.sh` lifts them (they used to be captured from a run
  with `SPU_RAW_DUMP_DIR`).
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
  raw SPU 1 (the second raw-SPU program), proxy-DMAs the program back into local
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
  `F9726FDA4A599084`, the latter with its parameter area masked; the policy
  module is byte-identical). New imports:
  `cellSaveDataAutoSave2`/`AutoLoad2`, `_sys_process_atexitspawn`,
  `cellPadSetActDirect` (rumble). At the exitspawn `port/stubs.cpp` runs the
  v1.7 build when it and its ELF exist (`RES_GAME_VERSION=100` keeps 01.00).
  `tools/v17/regen_v17.sh` (`tools/rebuild.sh v17`) extracts `patch_06`,
  decrypts `new_game.self` with `rpcs3 --decrypt`, lifts it, lifts the SPU
  programs (the raw-SPU ones cut out of the ELF) and builds. Verified:
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
- **Performance (2026-10-23):** the game samples its 2560×720 depth buffer
  as an A8R8G8B8 texture every frame; that was a GPU→CPU readback and a
  7.4 MB re-upload per frame and is now resolved on the GPU
  (`PS3RECOMP_ENG_ZRGBA_GPU=0` turns it off): RSX thread 54% → 12% busy,
  texture re-uploads per 120 frames ~100 → 0. A fast path for the PPU's
  store-watch check (it was 28% of the main thread) and guest clocks on
  `mach_absolute_time`. SPU: native single precision in round-toward-zero
  (bit-exact; `SPU_FP_NATIVE=0` off), a local `goto` for computed branches
  inside a function (`SPU_LOCAL_IB=0`), `SPU_DIRECT` calls and cheaper call
  returns, measured at about +5% (32.2/33.0 fps on vs 30.7 off, noisy). The
  campaign now reaches the game's own ~30–33 fps cap; the PPU's remaining
  "busy" time is largely the game's frame limiter. The hot SPU jobs run on
  one SPU because the game pins them there (workload 5, contention 1..1),
  the same as on the hardware.
- **The inFAMOUS 2 settings (2026-10-23, needs a play test):** keyboard and
  mouse on macOS (the window host records them for the game's cellPad; an
  FPS layout by default, remapped with `PS3_INPUT_MAP=<input.toml>`, the
  inFAMOUS 2 launcher's format; mouse speed, inversion, dead zone, capture;
  `PS3_INPUT_MODE`), rumble (`PS3_RUMBLE`, `PS3_RUMBLE_STRENGTH`), window
  modes, screen choice, vsync and MetalFX in the window host (which now
  letterboxes), SMAA (MIT shaders in `third_party/smaa`) and the present
  pass's filters and LUT, texture packs (`PS3_MODS`, `PS3_MODS_DUMP`) and the
  internal resolution (`PS3_RES_SCALE`, experimental). The toolkit tests
  pass; none of it has been verified in a play session yet. The launcher
  exposes them on its Controls and Graphics pages and passes each one only
  when it differs from the default.
- **Continue from a checkpoint (2026-10-23):** it froze on loading, on v1.7
  and 01.00. The second raw-SPU program carries a 0x50-byte parameter area
  (01.00: image+0x5680 of 0x56D0 bytes; v1.7: image+0x5780 of 0x57D0) that
  the PPU fills in before every load (heap pointers, per-level values, its
  raw SPU's MMIO window), so the GET run `spu_raw.c` fingerprints differs with
  the level, the checkpoint and the SPU number: a Continue from the player's
  checkpoint came in as `5771C48D28664765` (v1.7) / `78DE17DD3D838B86`
  (01.00), which nothing was registered under. The runtime now also matches a
  run with a registered image's parameter area zeroed
  (`spu_workload_register_img_masked`, toolkit patch, upstream #538); the port
  registers `1894F66385AFD052` plain and `2B2DB6A5DE233A5F` masked (01.00),
  `6D03497950639952` plain and `F9726FDA4A599084` masked (v1.7). Verified:
  Continue loads and plays on both; new campaign and respawn still work.
- **Movie sound (2026-10-23):** cellAdec was a stub that returned silence, so
  every movie was mute. It now decodes AC3 5.1 through the run-time FFmpeg
  (`libs/codec/adec_ffmpeg.c`; the PCM queue, bitstream info and timestamps
  in `cellAdec.c`; `test_adec_decode.c`). The POSIX mixer paces itself on the
  clock: one block per 5.333 ms, at most quarter-period nudges, 4 ms of grace
  for a late producer, 7.1 folded to stereo without clipping, each port's
  initial level honoured. `PS3_AUDIO_DUMP=<wav>` records the mix and
  `PS3_AUDIO_STATS=1` logs per-port levels. Verified: the intro movie's port
  at −25…−34 dBFS on all six channels, 0–0.2% silent blocks (it was silent);
  gameplay sound present.
- **Fog, smoke and the fake water (2026-10-23):** `WPOS` follows
  `SET_SHADER_WINDOW`, whose origin is bottom-left here, and the fragment
  decompiler ignored it. The ground-fog, smoke and one more shader look the
  depth buffer up at 720 − `WPOS.y`, so they read it upside down, and the
  river level showed a vertically streaked fake "reflection" over the dry
  creek bed (covering a body and an ammo crate). `rsx_fp_reads_wpos` /
  `rsx_fp_apply_wpos_window` apply the window in the programs that read
  `WPOS`, and the shader window is in their pipeline key
  (`test_fp_decompiler`, 112 pass; `test_rsx_draw_engine`). Swizzled
  (Morton-order) 16-bit depth shadow maps are now written back to guest
  memory, so distant soldiers no longer stand on dark rectangles. Verified on
  both campaigns.
- **A hidden window (2026-10-23):** a fullscreen window in its own Space stops
  presenting while another app is in front; that is macOS, not a freeze. The
  window host now logs it ("window off screen…", `hidden=1` in `[RSX FPS]`)
  and shows the newest frame on return.
- **The shadow-map pool (2026-10-24):** dark hard-edged rectangles on the
  ground. The draw engine keeps a table of colour surfaces and one of depth
  (zeta) surfaces, 64 slots each, and never evicted anything. Resistance
  packs its shadow maps into a per-frame pool whose addresses shift from
  frame to frame, so the tables filled up, a new shadow pass found no slot
  and was dropped without a word, and the receivers sampled an empty map. The
  tables now evict the least recently used entry
  (`libs/video/rsx_draw_engine.c`; the log says "surface table full: … takes
  the slot of …"; `test_surface_tables_full`). The evictions happen in a play
  session's log; the scene itself still needs a play test.
- **60 fps pacing (2026-10-24):** uncapped, the campaign flips on every vblank
  tick it gets. On the 62.5 Hz clock (16 ms ticks) a frame that takes 17 ms
  waits for the tick after next and stays up for 32 ms, which held the unlock
  to ~50–56 fps. With `RES_UNLOCK_FPS=1`, `port/patches.cpp` now sets the
  vblank to 120 Hz (`PS3_VBLANK_HZ=120` unless it is already set): 88.5 fps
  measured in gameplay, game speed unchanged (the patch moves the game to
  real time steps). `PS3_FLIP_MAX_HZ=<hz>` (toolkit, off by default) holds
  flips no closer together than 1/hz; capping at 60 measured worse (50 fps),
  because the game waits for the previous flip before its next frame.
- **Polygon offset, invariant positions, occlusion counts (2026-10-24):** the
  draw engine ignored the polygon offset, which the game uses on the
  character shadow maps, terrain decals and projected shadows; it is now
  passed to Metal (`PS3RECOMP_ENG_POLY_OFFSET=0` turns it off). The vertex
  position is compiled `invariant`, so the foliage's depth pre-pass and its
  `EQUAL` colour pass compute the same depth (`PS3RECOMP_MSL_INVARIANT=0`
  turns it off). Real ZPASS occlusion counts are implemented (Metal
  visibility results; `test_zpass_engine`, `test_metal_occlusion`) but off
  by default, `PS3RECOMP_ENG_ZPASS=1` turns them on: waiting for the counts
  cost 84 → 50 fps in 60 fps mode, one run crawled, and no visible gain was
  found.
- **Cooperative split-screen (2026-10-24):** player 1's (left) view was black.
  `CLEAR_SURFACE` ignored the scissor, so player 2's clear wiped player 1's
  half. Clears now go through new backend hooks (`clear_color_rect`,
  `clear_depth_stencil_rect`; partial-clear passes on Metal) when the scissor
  is narrower than the clip; a clip-only limit stays a whole clear (making
  the composite's per-frame clear partial cost a pass every frame).
  `PS3RECOMP_ENG_RECT_CLEAR=0` goes back to whole clears; `test_metal_rect_clears`.
  Verified in game: both halves render.
- **Robustness (2026-10-24):** over two hours of scripted play without a
  crash. Multiplayer → Online Game hung offline; the network-start dialog
  now answers "cancelled" when there is no network, and
  `sceNpManagerRequestTicket`/`RequestTicket2`/`GetTicket`/`GetTicketParam`
  are added (`test_netstart_offline`, `test_np_ticket_offline`). The unknown
  NID the game called at quit (`0x76948BFC`) is `cellSysconfAbort`, added with
  `cellSysconfOpen` (`test_sysconf`); `cellPadInfoSensorMode` and
  `cellPadSetSensorMode` are in (`test_pad_sensor_mode`). A full run now has
  no unresolved NIDs. Verified: every EBOOT menu and extra, offline
  split-screen multiplayer, pause and options, Save & Quit → Continue
  (exitspawn both ways), death and respawn at 30 and 60 fps, a fresh empty
  HDD, a corrupted `quick.sav`, a 25-minute 60 fps soak and a 22-minute idle
  one.

---

## What is missing to reach gameplay

### Blockers

- [x] **SPU jobs time out in gameplay.** Fixed 2026-10-22: the 15 WWS job modules lifted, SPURS kernel-style selection and atomic SPU events (see "Gameplay" under Done). The job manager loads job code into LS (around 0x7C00–0x14000 in its image) and the interpreter runs it at about 8–9 M steps/s. The jobs the first frame needs miss the game's 2 s timeouts. Capture each job's code as it is DMA'd in, lift it (inFAMOUS 2's `lift_spu_jobs.sh` / `relift_jobmod.py` are the model) and register it.
- [x] **Hang after the intro movie (intermittent).** Fixed 2026-10-22: a shared HLE→guest callback stack (see "Gameplay" under Done). A movie-player thread (`func_005F0360 → … → func_005EEFE4`) stays in `sys_mutex_lock`, so the player never reports done and the main thread stays in its load loop (`func_00044DE0`, state 2). `MUTEX_WATCH=2` keeps the lock blocking and reports waiters older than 5 s with the mutex owner; so far the hang has not reproduced under any watch mode. The fair-share mutex policy on macOS did not fix it.
- [x] **The player respawns above the map.** Fixed 2026-10-22: a raw-SPU stop and restart the runtime dropped (see "Current state"). After dying the checkpoint reload sent the player straight up into the sky (draw count ~30–40: sky, far terrain and water); the save was fine. In a test with 12 deaths and respawns the camera always came back to the first life's position (0 of 666 camera samples out of range, 193 of 292 before); a separate run confirmed it (back on the ground at the same spot, then walking normally).
- [x] **Dark rectangles on the ground (shadows) and the smoke.** Fixed 2026-10-22, gone in game (checked visually). The texture shadow comparison was in ARB order; the console and RPCS3 compare stored depth OP reference (RPCS3 issue #19391 shows the same rectangles). Fixed with the other RPCS3-comparison fixes (depth textures replicate depth to every lane, for soft particles and smoke; depth snapshots cropped to the texture's region).
- [x] **Frame rate.** ~30 fps in gameplay (2026-10-22): 32.5 fps with the inFAMOUS 2 stack vs 6.0 fps with its knobs off; 30.5 fps in the town with every fix in.
- [x] **Buildings and roofs render solid bright blue.** Fixed 2026-10-22: the 28-bit vertex offset wrap (see "Current state"); the town renders with its proper colours.
- [x] **Legal-screen text overlaps.** Fixed 2026-10-23: the runtime reported the licence area of a Japanese console, so the game drew the wrong region's legal screens over each other. It now reports North America (upstream [#429](https://github.com/sp00nznet/ps3recomp/pull/429); `PS3_LICENSE_AREA` overrides it); boot goes logos → title "Press the START button".
- [x] **Continue from a checkpoint froze on loading** (v1.7 and 01.00). Fixed 2026-10-23: the second raw-SPU program's parameter area is masked out of its fingerprint (see "Current state"); Continue from the player's checkpoint loads and plays on both campaigns.
- [x] **Every movie was silent.** Fixed 2026-10-23: cellAdec decodes AC3 5.1 through FFmpeg and the POSIX mixer paces itself on the clock; the intro movie at −25…−34 dBFS on all six channels.
- [x] **Fake water, fog and smoke on the river level.** Fixed 2026-10-23: `WPOS` with the shader window's bottom-left origin; the level shows the dry creek bed with soft fog (both campaigns).
- [x] **Distant soldiers stood on dark rectangles.** Fixed 2026-10-23: swizzled 16-bit depth shadow maps written back to guest memory.
- [x] **Dark hard-edged rectangles on the ground (the shadow-map pool).** Fixed 2026-10-24: the draw engine's surface and depth tables evict the least recently used entry instead of dropping new shadow passes once full (see "Current state"); the evictions show in a play session's log, the scene still needs a play test.
- [x] **Multiplayer → Online Game hung offline.** Fixed 2026-10-24: the network-start dialog answers "cancelled" without a network; NP ticket calls added.
- [ ] The job manager's idle lanes are halted after 30 s of polling the same queue line (`SPU_DMA_REPEAT_SECS`); check that halted lanes come back when work arrives.

### Into the first chapter

- [x] The level-1 intro movie, watched to the end (5,584 pictures).
- [x] Chapter 1 loading screen ("The Gauntlet", York) and "Press the START button".
- [x] The first gameplay frames in York (first person, HUD).
- [x] York played with a controller: enemies and allies, shooting, dying and respawning (2026-10-22).
- [x] Pause menu and in-game options (2026-10-24, scripted play).

### SPU jobs

- [ ] A capture path for the job manager's job code (the PM DMAs it in itself, so `SPU_DUMP_MISS` does not see it).
- [ ] A script that lifts every captured job in one go.
- [ ] The NextSynth audio jobs, which run in the interpreter from boot.

### Video and audio

- [ ] A/V sync over a long movie.
- [x] Movie sound: cellAdec decodes the AC3 5.1 tracks through FFmpeg (it returned silence) (2026-10-23).
- [ ] The menu music sometimes has extra centre/LFE content (+4–6 dB); it comes from the game's synth and varies from run to run.
- [x] Audio pacing on macOS: the mixer alternated between 2.4× and 0.95× real time; it now keeps real time (from the inFAMOUS 2 work, 2026-10-23). Volume, mute and output device: `PS3_AUDIO_VOLUME`, `PS3_AUDIO_MUTE`, `PS3_AUDIO_DEVICE`.
- [ ] Audio check by ear over a long windowed session (`AUDIO_WAV` to record).

### Menu

- [x] Title screen and main menu.
- [x] Saves: `PROFILE.SAV` and `quick.sav` written and read under `PS3_HDD0_ROOT`.
- [x] Every EBOOT menu and extra, and Options (2026-10-24, scripted play).
- [ ] `cellOskDialog` (on-screen keyboard), `cellMsgDialog`.

---

## What is missing to be 100% playable

### Gameplay

- [x] The first chapter (York) with a controller, in a window (2026-10-22; with the rendering bugs above).
- [ ] The first chapter (York) with keyboard and mouse (ported 2026-10-23, needs a play test).
- [ ] Play the whole campaign and write down what breaks: every chapter, the vehicles, the bosses, every weapon's alt-fire.
- [x] Offline split-screen (co-op and versus): both viewports render (2026-10-24; player 1's half was black, see Graphics). A session with two physical controllers is still to do.

### Graphics

- [ ] Compare against the original (RPCS3 or the console) chapter by chapter; open an item per bug with a capture, as in inFAMOUS 2.
- [x] The river level's fake water, ground fog and smoke (`WPOS` and the shader window) and the distant soldiers' shadows (2026-10-23).
- [x] Dark hard-edged rectangles on the ground: the surface and depth tables evict the least recently used entry, so the shadow-map pool's passes are no longer dropped (2026-10-24; play test of the scene pending).
- [x] Cooperative split-screen: player 1's view was black; clears honour the scissor (`PS3RECOMP_ENG_RECT_CLEAR=0` reverts) (2026-10-24).
- [x] The polygon offset (character shadow maps, terrain decals, projected shadows; `PS3RECOMP_ENG_POLY_OFFSET=0` off) and an invariant vertex position (the foliage's depth pre-pass and `EQUAL` colour pass; `PS3RECOMP_MSL_INVARIANT=0` off) (2026-10-24).
- [ ] Dark speckles on the foliage: it looks like the game's own content (grainy leaf alpha); to confirm against RPCS3.
- [ ] An eye glitch on a character, reported but not reproduced: it needs a repro.

### Performance

- [x] ~30 fps (the game's own target) in gameplay: 30.5 fps in the town with every fix in (2026-10-22). Keep measuring with every change.
- [x] The cross-frame vertex cache from the inFAMOUS 2 port: 5.9 → 14 fps (2026-10-22; `PS3RECOMP_ENG_VCACHE=0` turns it off).
- [x] Optimised builds: runtime -O2 and lifted PPU code -O1 in every configuration but Debug (it had been -O0) (2026-10-22).
- [x] An fps A/B of the inFAMOUS 2 performance stack: 32.5 fps on vs 6.0 fps with its knobs off (`PS3RECOMP_ENG_VCACHE_SPLIT=0`, `PS3RECOMP_METAL_BIND_ALL=1`, `PS3_FRAMECLOCK_US=4000`), same build, first 60 s of gameplay (2026-10-22).
- [x] The campaign at the game's own ~30–33 fps cap (2026-10-23): the GPU depth-as-RGBA8 resolve (RSX thread 54% → 12% busy, texture re-uploads per 120 frames ~100 → 0), the PPU store-watch fast path (it was 28% of the main thread), `mach_absolute_time` guest clocks, and SPU native single precision, local `goto` for computed branches, `SPU_DIRECT` and cheaper call returns (about +5%, 32.2/33.0 vs 30.7 fps, noisy). What is left of the PPU's "busy" time is largely the game's frame limiter; the hot SPU jobs are pinned by the game to one SPU (workload 5, contention 1..1), as on the hardware.
- [x] 60 fps pacing: with `RES_UNLOCK_FPS=1` the vblank runs at 120 Hz; 88.5 fps in gameplay (it was ~50–56), game speed unchanged (2026-10-24). `PS3_FLIP_MAX_HZ` caps flips (off by default; 60 measured 50 fps).
- [ ] Real ZPASS occlusion counts (`PS3RECOMP_ENG_ZPASS=1`): implemented, off by default; waiting for the counts cost 84 → 50 fps at 60 fps and no visible gain was found. Turn them on only if a scene needs them.
- [ ] A proper fps measurement of the v1.7 campaign (same A/B as 01.00).
- [ ] Measure how much SPU time is interpreted and cut it (translate what is left).

### Stability

- [x] Long sessions without a crash (2026-10-24): over two hours of scripted play, a 25-minute 60 fps soak and a 22-minute idle one.
- [ ] A `watch_stall.sh`-style hang report.
- [x] Dying and respawning at 30 and 60 fps (`RES_UNLOCK_FPS=1`) (2026-10-24).
- [x] Save & Quit → Continue (exitspawn both ways), a fresh empty HDD, a corrupted `quick.sav` (2026-10-24).
- [x] The unknown NID `0x76948BFC` the game calls at quit is `cellSysconfAbort`; added with `cellSysconfOpen` (2026-10-24). A full run has no unresolved NIDs.

### Controls

- [ ] Keyboard and mouse on macOS with a configurable mapping (`PS3_INPUT_MAP`): ported from the inFAMOUS 2 work and exposed in the launcher's Controls page (2026-10-23); tests pass, needs a play test.
- [x] DualSense/DualShock/Xbox via SDL2 (a DualSense played chapter 1, 2026-10-22).
- [ ] A physical controller with the launcher: the game process is in the background, so `SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1` is set; not tested with a controller yet.
- [ ] Rumble with a physical controller: the v1.7 `game.self` calls `cellPadSetActDirect`; rumble (`PS3_RUMBLE`, `PS3_RUMBLE_STRENGTH`) ported 2026-10-23, needs a play test.
- [ ] SIXAXIS: the game calls `cellPadSetSensorMode`/`cellPadInfoSensorMode` (both implemented 2026-10-24); find what it uses motion for and map it to a button.

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
- [x] The raw-SPU programs and the SPU jobs from the disc without a capture run: the job modules and the raw-SPU programs are cut out of the ELFs (`tools/regen_jobmods.sh`, `tools/extract_rawspu_elf.py`, 2026-10-23).
- [ ] Decryption without RPCS3 (inFAMOUS 2's `decrypt_self.py`, with the player's own key file).

---

## After it is playable (improvements)

### Graphics and resolution

- [ ] Internal resolution above 720p (`PS3_RES_SCALE`, experimental), MetalFX (`PS3_UPSCALE=metalfx`), fullscreen/borderless (`PS3_WINDOW_MODE`, `PS3_DISPLAY`, `PS3_VSYNC`), SMAA and filters, texture packs: ported from the inFAMOUS 2 work (2026-10-23), tests pass, needs a play test.
- [x] Past 30 fps and a wider field of view (2026-10-23): the community's RPCS3 patches (Whatcookie, FlexBy, illusion, Score) carried over to the static recompilation. The data half is written after the ELF loads; the code half is lifted in both forms (`ppu_lifter.py --insn-patches`) and picked at run time, and each set applies only when 7–10 known words match the running ELF. `RES_UNLOCK_FPS=1`: ~56 fps in the campaign on an idle machine, 01.00 and v1.7, game speed normal in wall-clock time (the intro camera move takes 14.4–16.4 s at 33, 56 and 100 fps; walking ~6.3 units/s); `PS3_VBLANK_HZ=120` ~100 fps. `RES_MENU_60FPS=1`: menus 32 → 59 fps. `RES_FOV=<0.1..4>` (the game's 1.0; RPCS3's "Extended FOV" 0.7). The launcher's "60 fps" switch passes both fps patches. Since 2026-10-24 the unlock runs the vblank at 120 Hz: 88.5 fps in gameplay.

### App to install and play (launcher)

- [x] The window keeps its position, size and fullscreen across the menu → campaign exitspawn (`PS3RECOMP_WINDOW_STATE`, 2026-10-22).
- [x] One persistent window (2026-10-22): the `Resistance` launcher (`tools/host_window/host_window.m`) owns the window and the menu and campaign executables draw into IOSurfaces shared with it; no window or Dock icon of the game's own. Closing the window ends the game; if the launcher dies the game exits within ~2 s; without it the backend opens its own window. Verified from the menus through the campaign intro at ~33 fps.
- [x] The window title's guest FPS counts each flip (it read 0 with the window host) (2026-10-22).
- [x] The launcher app (`launcher/`, Tauri, 2026-10-22): ISO pick and mount, play folder, port-folder detection, campaign version (v1.7/01.00), settings (graphics, performance, game), a saves manager (here and in RPCS3), the game log, debug mode, an updates check, a Desktop and Steam shortcut, English and Brazilian Portuguese. Stop works: the window host now exits when the game does.
- [x] Controls page (bindings for keyboard, mouse and controller written to `input.toml`, the controller test, input mode, mouse, rumble) and Graphics settings (window mode, screen, vsync, MetalFX, internal resolution, SMAA, filters and LUT, texture packs and their dump), in English and Brazilian Portuguese; every setting passed only when it differs from the default (2026-10-23; the game side needs a play test).
- [x] Release v0.1.0 (2026-10-23, pre-release): the macOS launcher and a Windows preview.
- [x] Release v0.2.0 (2026-10-23, pre-release): the launcher with the Controls, Graphics and Audio settings, 60 fps and FOV.
- [x] Release v0.2.1 (2026-10-24, pre-release): the 60 fps pacing, split-screen and shadow fixes; the launcher's "60 fps" switch gets the port's 120 Hz vblank.
- [x] Audio settings (volume, mute, output device) and one "60 fps" switch (2026-10-23).
- [ ] A physical controller with the launcher (see Controls).
- [ ] Icon from the player's own `ICON0.PNG`, Game Mode app bundle on macOS.

### Other platforms

- [ ] Linux (Vulkan) and Windows (D3D12), with the launcher and the rebuild running natively. The Windows XInput runtime loading (upstream #494) is in the toolkit patch, untested on Windows; so is the audio mixer's new pacing.

### Out of scope for now

- [ ] Online (40-player matches, `sceNp`, `sys_net`, `cellMic` voice chat): the official servers are gone; only with a community server. Multiplayer → Online Game now backs out offline instead of hanging (2026-10-24).

---

## Done and finished

### Boot and pipeline

- [x] Disc dump `BCUS98107` v01.00, mounted as the VFS root.
- [x] `EBOOT.BIN` and `game.self` decrypted, analysed (EBOOT 25,022 functions, 183 imports; `game.self` 18,994 functions, 134 imports) and lifted; both build on macOS/arm64 with Metal.
- [x] Job-manager policy module lifted at LS base 0xA00 (SPU image 1).
- [x] Raw-SPU programs `1894F66385AFD052` and `2B2DB6A5DE233A5F` (parameter area masked) lifted at LS base 0x60 (SPU images 2 and 3).

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

- [x] The v1.7 `game.self` (`patch_06` `/data/new_game.self`) re-lifted and built as `build/BCUS98107_game_v17`; 13 of the 15 job modules and both raw-SPU programs re-lifted (`6D03497950639952`, `F9726FDA4A599084` masked); the policy module unchanged.
- [x] The exitspawn runs the v1.7 build when it and its ELF exist (`RES_GAME_VERSION=100` keeps 01.00); menus → New campaign → York intro, gameplay and respawn verified.
- [x] `tools/v17/regen_v17.sh` (`tools/rebuild.sh v17`), `tools/v17/psarc_extract.py`; `tools/play.sh` says which campaign runs and refuses to start with the update's `patch_NN.bin` in the play folder.
- [x] Shadows and smoke fixed in game (checked visually).
- [x] The launcher app (`launcher/`), with a working Stop (the window host reaps the game reliably).
- [x] The title's guest FPS counts each flip.

### Performance and the inFAMOUS 2 settings (2026-10-23)

- [x] GPU depth-as-RGBA8 resolve: RSX thread 54% → 12% busy, texture re-uploads per 120 frames ~100 → 0.
- [x] PPU store-watch fast path, `mach_absolute_time` guest clocks; SPU native single precision (bit-exact), local `goto` for computed branches, `SPU_DIRECT`, cheaper call returns (about +5%). The campaign at the game's own ~30–33 fps cap.
- [x] Keyboard and mouse, rumble, window modes, vsync, MetalFX, SMAA, filters and LUT, texture packs and the internal resolution ported from the inFAMOUS 2 work; tests pass (a play test is still to do, see Controls and Graphics).
- [x] The launcher's Controls page and new Graphics settings; v0.1.0 released.

### 60 fps, legal screens, audio and upstream fixes (2026-10-23)

- [x] The 60 fps / FOV patches (`port/patches.cpp`, `port/patches/*.insn.json`; `tools/rebuild.sh lift` and `tools/v17/regen_v17.sh` pass `--insn-patches`, `tools/relift.sh` re-lifts an older lift).
- [x] The legal-screen overlap (licence area, #429).
- [x] From upstream pull requests: vertex-program branch bits (#485, plus a program start-slot fix; 19 of Resistance's 343 vertex programs branch), `WPOS` on 2× AA targets (#446, with the internal resolution), the fs fd-table race and read/write switch (#430, #456), the `stwcx.` re-check (#522), linear ARGB fast decode (#432), the texture-slot memo (#438), the SPU decrementer on the raw clock (#451), the yield on repeated SPU DMA (#448), small SPU DMA savings (#453–#455), `getenv` once per call site (#515), Windows XInput loaded at run time (#494). From the inFAMOUS 2 work: audio pacing, volume and device, the opt-in `usleep` spin (`PS3_USLEEP_SPIN_US`). A gameplay regression run with all of them: renders, no errors, respawn OK.

### Continue, movie sound, fog and water (2026-10-23)

- [x] Raw-SPU programs with a parameter area: the masked fingerprint (`spu_workload_register_img_masked`, `test_spu_workload_masked.c`), the masked registrations for 01.00 and v1.7, `tools/extract_rawspu_elf.py` (no capture run). Continue from a checkpoint loads and plays on both campaigns.
- [x] Movie sound: cellAdec's AC3 5.1 decode through FFmpeg (`libs/codec/adec_ffmpeg.c`, `test_adec_decode.c`) and the POSIX mixer's pacing (one block per 5.333 ms on the clock, 7.1 → stereo without clipping, each port's initial level).
- [x] `WPOS` with the shader window's origin (the river level's fake water, ground fog and smoke) and swizzled 16-bit depth shadow maps written to guest memory (the distant soldiers' shadows); verified on both campaigns.
- [x] The window host says when the window is hidden or off screen and shows the newest frame on return.

### Shadows, 60 fps pacing, split-screen and robustness (2026-10-24)

- [x] The draw engine's surface and depth tables evict the least recently used entry (the shadow-map pool; "surface table full: … takes the slot of …"; `test_surface_tables_full`).
- [x] The 120 Hz vblank under `RES_UNLOCK_FPS=1` (`port/patches.cpp`; 88.5 fps in gameplay) and `PS3_FLIP_MAX_HZ` (off by default).
- [x] The polygon offset, an invariant vertex position, and ZPASS occlusion counts (off by default, `PS3RECOMP_ENG_ZPASS=1`).
- [x] Scissored clears through `clear_color_rect` / `clear_depth_stencil_rect` (Metal partial-clear passes): both split-screen halves render.
- [x] Offline answers for the network-start dialog and the NP tickets (Online Game no longer hangs), `cellSysconfAbort` / `cellSysconfOpen`, `cellPadInfoSensorMode` / `cellPadSetSensorMode`; a full run has no unresolved NIDs. Over two hours of scripted play without a crash.

### Tools

- [x] `RES_PEEK` (guest memory peeks from `port/stubs.cpp`), `MUTEX_WATCH=1|2`, `VDEC_PICLOG`, `PPU_STACKINFO_LOG`, `SPU_RAW_DUMP_DIR`.
- [x] Headless runs with frame dumps and scripted input (`tools/run.sh`, `start.sh`, `press.sh`, `snap.sh`, `to_campaign.sh`).
- [x] `RSX_ENGINE_PASS_DUMP_TRIGGER=<file>`: the per-pass dump starts when that file appears, so a script captures the frame it has steered the game to (2026-10-22).
- [x] `PAD_FILE` stick commands, `S lx ly rx ry polls`: hold the sticks from a script to walk or turn the camera (2026-10-22).
- [x] `RSX_ENGINE_PASS_DUMP_DRAWS` logs each draw's vertex attribute formats and first bytes; `RSX_VPC_LOG=<constant>[,<draw>]` logs a vertex-program constant every 30 frames (`c16` = camera position in the world shaders) (2026-10-22).
- [x] `PS3_AUDIO_DUMP=<file.wav>` records the final mix for a whole session; `PS3_AUDIO_STATS=1` logs each audio port's and the mix's level, silent blocks and clipping every ~5 s (2026-10-23).
- [x] `tools/extract_rawspu_elf.py`: the raw-SPU programs cut out of `game.elf`, the parameter area zeroed, with a JSON manifest (2026-10-23).

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
- [ ] [#509](https://github.com/sp00nznet/ps3recomp/pull/509) macOS: one window across an exitspawn (`tools/host_window`, Metal host-surface mode); follow-up: the window host says when the window is off screen and shows the newest frame on return.
- [ ] [#538](https://github.com/sp00nznet/ps3recomp/pull/538) spu_workload: raw-SPU images with a parameter area the loader fills in (the Continue freeze; stacked on #473).
- [ ] [#539](https://github.com/sp00nznet/ps3recomp/pull/539) ppu_lifter: `--insn-patches` lifts patched instructions as run-time switches (the 60 fps patches).
- [ ] [#540](https://github.com/sp00nznet/ps3recomp/pull/540) sys_timer: one host clock for `mftb`, system time and timers (`mach_absolute_time` on macOS).
- [x] Its own repository (`resistance-recomp`): port sources, a toolkit patch, `tools/setup_toolkit.sh`, README, credits, and a `.gitignore` that blocks game data, firmware and saves.
