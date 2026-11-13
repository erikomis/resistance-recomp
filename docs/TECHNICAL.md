# Resistance: Fall of Man port — technical notes

🌐 **English** · [Português (BR)](TECHNICAL.pt-BR.md)

> The player-facing README is [../README.md](../README.md). This page keeps the full status table, how the port works, building step by step, the environment variables and the layout.

A native port of **Resistance: Fall of Man** (PS3, disc `BCUS98107`, v01.00)
to **macOS/arm64**, built with static recompilation: the game's PowerPC and
SPU code is translated to C/C++ and compiled into native executables, with
graphics on Metal. It is not an emulator.

**It is playable: the campaign has been played with a controller from York
through Grimsby into Manchester ("The Cathedral"), and every one of its 30
levels loads to gameplay.** The front end runs from the legal screens through
the first-run install, the title screen and the main menu to a new campaign.
The campaign executable then plays the level-1 intro movie and loads "The
Gauntlet" (York). In a window, with a DualSense, the campaign plays in first
person with the HUD: enemies and allies, shooting, dying and respawning from
the checkpoint, Save & Quit and Continue. It runs at about 29–33 fps in play,
the game's own 30 fps target (about 88 fps in gameplay with the optional
60 fps patch), in one window from the menus through the campaign; offline
split-screen draws both players' views. The campaign is the game's last update, v1.7, re-lifted from its
`patch_06`; the menus stay the disc's 01.00 (see [Status](#status)).

Built on [ps3recomp](https://github.com/sp00nznet/ps3recomp), following the
author's [inFAMOUS 2 port](https://github.com/erikomis/infamous2-recomp). See
[CREDITS.md](../CREDITS.md).

> [!WARNING]
> **Work in progress.** The campaign can be played well into Manchester,
> with rendering bugs still open. The [releases](#releases--download) carry only the launcher:
> you build the game yourself from your own disc (the [launcher](#launcher)
> can run the rebuild and the game for you).
> Apple-silicon macOS only. A controller is the tested way to play; keyboard
> and mouse are new and still need a play test. What is missing is in
> [ROADMAP.md](../ROADMAP.md).

> **This repository contains no game data, no firmware, no Sony keys and
> nothing translated from them.** Everyone generates all of that locally from
> their own legitimate copy. To decrypt the disc's executables, the rebuild
> downloads one public file, RPCS3's key table (`rpcs3/Crypto/key_vault.cpp`
> at a pinned commit, checked by its SHA-256), and keeps the keys it needs in
> a file of your own outside the repository (`tools/fetch_keys.py`). A fan
> project, not affiliated with Sony Interactive Entertainment or Insomniac
> Games.

## Releases / Download

The [releases page](https://github.com/erikomis/resistance-recomp/releases)
has the [launcher](#launcher) for Apple-silicon macOS (a zipped `.app` and a
`.dmg`), the toolkit patch and the sources. **The game is not in it**: the
port's executables are code converted from the game, so you build them from
your own disc ([Building from source](#building-from-source)). The builds are
unsigned: right-click the app and choose Open the first time.

A **Windows x64 launcher** (NSIS `.exe` and `.msi`) is published as a
**preview**: it compiles and starts, mounts an ISO and runs
`build\BCUS98107\ResistanceRecomp.exe`, but the port itself has never been
built or played on Windows, so treat it as experimental and untested
(SmartScreen may warn; "More info" > "Run anyway"). Details in
[launcher/README.md](../launcher/README.md#windows-preview).

The latest release is **v0.2.2** (2026-10-29, a pre-release): the macOS
launcher and the Windows preview, and the toolkit patch with the fixes that
let the campaign be played on: the Grimsby freeze ("Fates Worse than Death",
the system cache path), Campaign → Location Select no longer quitting the
game (Lua's error handling, the lifter's `--sjlj`), `RES_LEVEL` to start any
level, and the level sweep. A lift made before it needs `tools/relift.sh`
(see [Building](#building)). v0.2.1 (2026-10-24) brought the 60 fps pacing
(120 Hz vblank, ~88 fps in gameplay), the cooperative split-screen fix, the
shadow-map pool fix on the ground and the offline and quit-time fixes. v0.2.0 (2026-10-23) added the controls, window modes, filters, texture
packs, 60 fps and field of view and audio settings. To build the launcher
from the sources instead:
`tools/rebuild.sh launcher`.

## Status

As of 2026-10-29 (played by hand from York into Manchester's "The Cathedral" at about 29–33 fps; all 30 campaign levels load to gameplay; the Grimsby freeze and the Location Select quit fixed; earlier: 60 fps and field-of-view options; the legal-screen overlap fixed; audio pacing, volume and output device; 18 toolkit fixes from upstream pull requests; Continue from a checkpoint, movie sound, the river level's fog and fake water and the distant soldiers' shadows fixed; then the 60 fps pacing, the cooperative split-screen, the dark rectangles on the ground, the polygon offset and two hours of scripted play without a crash; after v0.2.1, Save & Quit no longer takes the 120 Hz vblank back to the menus), every ✅ item verified in runs on an Apple-silicon Mac with Metal; 🧪 = built and tested, not yet verified in a play session.

| | |
| --- | --- |
| Both executables lifted to C++ and built (`EBOOT.BIN`: 25,022 functions; `game.self`: 18,994), and the v1.7 update's `game.self` (19,706 lifted functions, 19,219 for 01.00's) | ✅ |
| Boot, legal screens | ✅ logos → title "Press the START button", no overlapping legal text (the runtime reported a Japanese console's licence area; it now reports North America, `PS3_LICENSE_AREA` overrides it) |
| First-run "initial setup" HDD install (writes the `*.done` cache files) | ✅ |
| Title screen, main menu (Campaign / Cooperative / Multiplayer / Options) | ✅ |
| Campaign difficulty and invert-Y prompts | ✅ |
| Saves and profile (`quick.sav`, `PROFILE.SAV`) written and read | ✅ |
| EBOOT → `game.self` hand-off (`sys_game_process_exitspawn`) into the level-1 intro movie | ✅ |
| Movies (cellVdec through FFmpeg, drawn as an immediate-mode quad) | ✅ the level-1 intro plays to the end, with its sound: cellAdec decodes the AC3 5.1 tracks through FFmpeg (it had been a stub that returned silence, so every movie was mute) |
| Raw SPUs (two programs `game.self` starts when a level loads) | ✅ cut out of the ELF (no capture run) and lifted; Continue from a checkpoint loads and plays (it froze: the program carries a parameter area that changes with the level and checkpoint, now masked out of its fingerprint) |
| Chapter 1 ("The Gauntlet", York) loading screen | ✅ up to "Press the START button" |
| Gameplay | ✅ played in a window with a DualSense from York (The Gauntlet, A Lone Survivor, Spires) through Grimsby (Fates Worse than Death, Conversion, Hunted Down) and Manchester's Path of Least Resistance into "The Cathedral" (2026-10-29): first person, HUD, enemies and allies, shooting, dying and respawning, Save & Quit and Continue |
| Every campaign level | ✅ a sweep loaded all 30 levels (York to the Chimeran Tower, `RES_LEVEL=<n>`) to gameplay with no errors, unresolved calls or SPU timeouts (2026-10-25); the table is in [ROADMAP.md](../ROADMAP.md#level-sweep-2026-10-25) |
| Grimsby freeze | ✅ "Fates Worse than Death" froze at the conversion-centre switch: the game stashes a cutscene in the system cache (`/dev_hdd1`) and its file layer finds that device only when `cellSysCacheMount` answers `/dev_hdd1` itself, as the console does; the runtime answered `/dev_hdd1/cache/<id>`, so the open failed and the main thread retried it for ever. Fixed 2026-10-25 (the same host folder, old caches keep working) |
| Campaign → Location Select | ✅ it quit the game: Lua's `pcall`/`error` use newlib's `setjmp`/`longjmp`, which a static recompilation cannot run as lifted code (restoring the guest registers unwinds no host frame), so a Lua error fell into the "no handler" panic. The lifter's `--sjlj` now turns those calls into host `setjmp`/`longjmp` (`port/patches/*.sjlj.json`, `runtime/ppu/ppu_sjlj.c`) (2026-10-25) |
| Frame rate | ✅ about 29–33 fps in hand play through the later levels too (2026-10-29: "The Cathedral", one of the three heaviest levels, held 29–32 fps at the game's 30 fps cap; the sweep's uncapped figures are in [ROADMAP.md](../ROADMAP.md#level-sweep-2026-10-25)); the campaign reaches the game's own ~30–33 fps cap (Resistance runs at 30 fps on the PS3); the PPU's remaining "busy" time is largely the game's frame limiter. Since 2026-10-22: the game's 2560×720 depth buffer, which it samples as A8R8G8B8 every frame, is resolved on the GPU (it was a GPU→CPU readback and a 7.4 MB re-upload per frame): RSX thread 54% → 12% busy, texture re-uploads per 120 frames ~100 → 0; see [How it works](#how-it-works) |
| 60 fps and a wider field of view | ✅ optional, the community's RPCS3 patches carried over to the port (`port/patches.cpp`): `RES_UNLOCK_FPS=1` 88.5 fps measured in gameplay (01.00 and v1.7; it was ~50–56 before the port ran the vblank at 120 Hz with it, 2026-10-24) at the normal game speed (the intro camera move takes 14.4–16.4 s at 33, 56 and 100 fps; walking ~6.3 units/s), `RES_MENU_60FPS=1` menus 32 → 59 fps, `RES_FOV=<0.1..4>` (the game's 1.0; RPCS3's "Extended FOV" 0.7); dying and respawning verified at 30 and 60 fps; after v0.2.1 (on main) the 120 Hz rate stays inside the campaign process, so Save & Quit no longer takes it back to the menus |
| Audio | ✅ the macOS mixer keeps real time (it alternated between 2.4× and 0.95×; it now mixes one block per 5.333 ms on the clock and folds 7.1 to stereo without clipping); movie sound (see Movies); 🧪 the menu music's centre/LFE that varied from run to run (+4–6 dB) was a race in the game: its speaker-count global starts at 2 and the audio-out setup sets it to 6 on another thread, so a sound setup that read it first mixed the whole session in the 2ch layout; the getter now answers 6 (2026-11-06, from the Resistance PC port by Psyykosis; `RES_SPEAKERS=0` reverts), not yet checked by ear; 🧪 `PS3_AUDIO_VOLUME=0..100`, `PS3_AUDIO_DEVICE=<name>`, `PS3_AUDIO_MUTE=1` (also in the launcher) |
| World colours (buildings, roofs, streets) | ✅ correct: the blue world was a vertex-offset wrap the RSX does and the port did not (see [How it works](#how-it-works)) |
| Shadows and smoke | ✅ the dark rectangles on the ground and the hard-edged smoke are gone in game (checked visually) after the depth fixes from a comparison with RPCS3: the shadow depth comparison ran in the wrong direction, depth textures now feed every lane (soft particles, smoke), depth snapshots are cropped to the region the texture names; distant soldiers no longer stand on dark rectangles (swizzled 16-bit depth shadow maps are now written back to guest memory); 🧪 the dark hard-edged rectangles that came back on the ground once the draw engine's surface tables were full (the shadow-map pool) are fixed by evicting the least recently used surface, seen working in a play session's log, the scene still to be re-checked in a play test; the polygon offset is honoured (character shadow maps, terrain decals, projected shadows) |
| Water, ground fog and smoke | ✅ the river level shows the dry creek bed with soft fog instead of a vertically streaked fake "reflection": the fragment shaders' `WPOS` ignored the window's bottom-left origin, so the fog and smoke shaders read the depth buffer upside down (verified on both campaigns) |
| Foliage | ✅ the grass layer on the ground and rocks draws (2026-11-06): the alpha test now compares the alpha the colour buffer stores (clamped, 8-bit) against the 8-bit ref, as the RSX does; the grass returns 0.5 − coverage and tests EQUAL 0, so compared raw nearly all of it was discarded (from the Resistance PC port by Psyykosis; `PS3RECOMP_FP_ALPHA_QUANT=0` reverts; seen in level 21); ⚠️ dark speckles in the trees still to re-check; it looks like the game's own content (grainy leaf alpha), not yet compared with RPCS3. The vertex position is now compiled invariant, so the depth pre-pass and the `EQUAL` colour pass agree |
| Split-screen | ✅ offline split-screen (co-op and versus) draws both views: player 1's (left) half was black because a clear ignored the scissor and player 2's clear wiped it (2026-10-24); a clear is now partial only when its scissor is narrower than the clip, so the full-screen composite pass stays whole (2026-10-25) |
| Respawn | ✅ after dying the player comes back on the ground where the checkpoint is (it used to be sent up into the sky; a raw-SPU restart the runtime dropped, see [How it works](#how-it-works)); verified over 12 deaths, and at 30 and 60 fps |
| Window | ✅ one persistent window from the menus through the campaign: the small `Resistance` launcher owns it and the game draws into it through shared IOSurfaces, so the menu → campaign hand-off no longer closes it (~0.35 ms GPU per frame, no CPU copy); the title's guest FPS counts the game's flips. A fullscreen window in its own Space stops presenting while another app is in front (that is macOS, not a freeze): the log says so ("window off screen", `hidden=1` in `[RSX FPS]`) and the newest frame is shown on return; closing the window, or Stop in the launcher, ends the game process in about 0.1 s (2026-11-06, from the Resistance 2 port): SDL had taken SIGTERM over and only posted an event nothing read, so the game outlived its window until the launcher killed it 3 s later; it now waits for a save in progress (up to 2 s) and exits (`PS3_CLEAN_EXIT=0` reverts) |
| SPU jobs (geometry, draw sort, effects, NextSynth audio) | ✅ the 15 WWS job modules lifted |
| After the intro movie | ✅ the old intermittent hang is fixed (shared callback stack) |
| Controller | ✅ DualSense, DualShock or Xbox through SDL2; Xbox One/Series pads are read as plain HID pads (2026-10-30, from the Resistance 2 port: through Apple's GameController/MFi path an Xbox Series pad showed as connected but never sent a button, since that input only arrives through a Cocoa run loop the game process does not run; `SDL_JOYSTICK_MFI=1` brings it back), not yet tried with a physical Xbox pad (input with the game process in the background is enabled for the launcher; not yet tested with a physical controller that way); 🧪 rumble (`PS3_RUMBLE`, `PS3_RUMBLE_STRENGTH`) ported, needs a play test |
| Keyboard and mouse | 🧪 ported from the inFAMOUS 2 work: an FPS layout (WASD, mouse look, left click fires), remappable in `input.toml` (`PS3_INPUT_MAP`, the launcher's Controls page); tests pass, needs a play test |
| Window modes, vsync, MetalFX, SMAA and filters, texture packs, internal resolution | 🧪 ported from the inFAMOUS 2 work (see [Environment](#environment)); tests pass, needs a play test; the internal resolution is experimental |
| Game update v1.7 | ✅ the campaign runs the update's `game.self`, re-lifted from `patch_06`: the hand play from York into Manchester was on v1.7, at 29–33 fps; the menus stay the disc's 01.00 EBOOT (see [The v1.7 campaign](#the-v17-campaign)); rumble with a physical controller not verified yet |
| Launcher app | ✅ [`launcher/`](../launcher/) (Tauri): disc, builds, rebuild, settings, saves, Play/Stop, in English and Brazilian Portuguese; 🧪 the new Controls page (remapping, mouse, rumble) and Graphics settings (window, filters, texture packs) |
| Linux and Windows | ⏳ not supported |
| Menus, saves and long sessions | ✅ over two hours of scripted play without a crash (2026-10-24): every EBOOT menu and extra, pause and options, Save & Quit → Continue (exitspawn both ways), a fresh empty HDD, a corrupted `quick.sav`, a 25-minute 60 fps soak and a 22-minute idle one; a full run has no unresolved NIDs (the one at quit was `cellSysconfAbort`) |
| Online multiplayer | ⏳ future, through the community's [PS Rewired](https://psrewired.com) servers ([plan](../ROADMAP.md#online-multiplayer-through-ps-rewired-future)); the official servers are gone; Multiplayer → Online Game now backs out offline instead of hanging |

Next: play on from Manchester to the end of the campaign; speed up the three heaviest levels ("The Cathedral", "No Way Out", "Ice and Iron", ~25–26 fps uncapped in the sweep). Details and next steps in [ROADMAP.md](../ROADMAP.md).

## How it works

Resistance ships two PPU executables on the disc. `EBOOT.BIN` is the front end
(legal screens, install, menus, multiplayer); `game.self` is the campaign. On
the console, picking a campaign difficulty makes the EBOOT call
`sys_game_process_exitspawn`, which replaces it with `game.self`. Here each
executable is lifted and built on its own, and the port implements exitspawn
(`port/stubs.cpp`) as an `execv` of the other build, passing the guest's boot
path and arguments on and placing the parent's data block at
main-stack-top − 0x1000, where `game.self` looks for its `"ELFtastic!"`
block.

The fixes that got it this far:

- **Job manager:** the title's SPURS job-manager policy module (fingerprint
  `4E4076E700C6AFF3`, 11,136 bytes) is lifted at local-store base 0xA00, where
  SPURS loads it ([tools/regen_spu_pm.sh](../tools/regen_spu_pm.sh)).
- **Guest off the macOS main thread** (`port/main.cpp`): AppKit only runs on
  the main thread, and with the guest there the Metal backend's event pump
  deadlocked. The guest runs on its own thread and the main thread turns the
  run loop, as in the inFAMOUS 2 port.
- **Main-thread stack** (toolkit patch, `runtime/syscalls/sys_ppu_thread.c`):
  `sys_ppu_thread_get_stack_information` reported 0 + 0 for the main thread,
  which sent `game.self` looking for its exitspawn block at 0xFFFFF000.
- **Video:** the toolkit's cellVdec with FFmpeg loaded at runtime
  (`-DPS3RECOMP_FFMPEG_RUNTIME=ON`, Homebrew FFmpeg).
- **The blue world:** the RSX adds `VERTEX_DATA_BASE_OFFSET` to each vertex
  array's offset and wraps the sum at 28 bits, as RPCS3 does and Insomniac's
  engines rely on. Resistance's per-frame world lighting stream sits at
  base + offset = 0x1Fxxxxxx, which is local memory 0x0Fxxxxxx; without the
  wrap the port read past VRAM, got zeros and lit the world blue
  (`rsx_dsp_resolve_vertex_offset`, toolkit patch, with a regression test).
- **Speed:** a cross-frame vertex cache (5.9 → 14 fps), an optimised build
  (the port had been compiled at -O0; now the runtime is -O2 and the lifted
  code -O1 in every configuration but Debug) and the inFAMOUS 2 port's
  performance work (per-stream vertex cache, Metal staging ring, binding only
  changed state, program memo, MSL shader cache, a 1 ms frame clock with one
  present per flip): 32.5 fps in gameplay with it vs 6.0 fps with its knobs
  off, 30.5 fps in the town with every fix in.
- **Respawn:** on every checkpoint reload the game writes `SPU_RunCntl`=0 to
  raw SPU 1 (the second raw-SPU program), proxy-DMAs the program back into
  local store and writes `SPU_RunCntl`=1. The runtime ignored the stop while
  the SPU was parked in a channel wait and dropped the restart, so the old
  program kept running over the reloaded image and the player was sent into
  the sky. Now, as in RPCS3's `RawSPUThread.cpp`, a stop really stops the raw
  SPU (honoured in channel reads and waits and the out-mailbox stall; the stop
  waits for the host thread) and the restart starts it fresh
  (`runtime/spu/spu_raw.c`, toolkit patch). In a test with 12 deaths the
  camera came back to the first life's position every time (0 of 666 samples
  out of range, 193 of 292 before).
- **One window:** exitspawn is an `execv`, and a window goes away with the
  process image that opened it. A small launcher, `Resistance`
  (`tools/host_window/host_window.m` in the toolkit, target `Resistance` in
  `port/CMakeLists.txt`), owns the window and runs `ResistanceRecomp` with
  `PS3RECOMP_HOST_SURFACES` set to a Mach bootstrap name; the Metal backend
  then renders on the GPU into IOSurfaces it shares with the launcher and opens
  no window or Dock icon of its own. The image the exec brings up keeps the
  pid and attaches again. Closing the window ends the game; if the launcher
  dies the game exits within ~2 s; without the launcher the backend opens its
  own window as before.
- **Performance** (2026-10-23): the game samples its 2560×720 depth
  buffer as an A8R8G8B8 texture every frame. That was a GPU→CPU readback and
  a 7.4 MB re-upload per frame; it is now resolved on the GPU
  (`PS3RECOMP_ENG_ZRGBA_GPU=0` turns it off): the RSX thread went from 54% to 12% busy and
  texture re-uploads per 120 frames from ~100 to 0. The PPU's store-watch
  check has a fast path (it was 28% of the main thread) and the guest clocks
  read `mach_absolute_time`. In the SPU code: native single-precision
  arithmetic in round-toward-zero (bit-exact; `SPU_FP_NATIVE=0` turns it
  off), a local `goto` for computed branches inside a function
  (`SPU_LOCAL_IB=0`), direct calls to known lifted targets (`SPU_DIRECT`) and
  cheaper call returns, measured at about +5% (32.2/33.0 fps on vs 30.7 off,
  noisy). The campaign now reaches the game's own ~30–33 fps cap; the PPU's
  remaining "busy" time is largely the game's frame limiter. The hot SPU jobs
  run on one SPU because the game pins them there (workload 5, contention
  1..1), as on the hardware.
- **The v1.7 campaign:** the update's `patch_06` carries a new `game.self`.
  It is lifted like the disc's and built as `build/BCUS98107_game_v17`; at
  the exitspawn `port/stubs.cpp` runs it instead of the 01.00 build when it
  and its ELF exist. 13 of the 15 WWS job modules and both raw-SPU programs
  changed and are lifted from the v1.7 ELF; the policy module is the same.
- **Continue from a checkpoint** (it froze on loading): the second raw-SPU
  program carries a 0x50-byte parameter area the PPU fills in before every
  load (heap pointers, per-level values, its raw SPU's MMIO window), so the
  bytes it is loaded with, and their fingerprint, change with the level, the
  checkpoint and the SPU number; a Continue loaded one no build had
  registered. The runtime now matches a program with that area masked out
  (`spu_workload_register_img_masked`, toolkit patch), and both programs are
  cut out of the ELF by the `0xDEADDEAD` header the loader uses
  (`tools/extract_rawspu_elf.py`), so no capture run is needed.
- **Movie sound:** cellAdec was a stub that decoded nothing, so every movie
  was silent. It now decodes the movies' AC3 5.1 through the same run-time
  FFmpeg as the video (`libs/codec/adec_ffmpeg.c`, with its queue, bitstream
  info and timestamps), and the POSIX mixer paces itself on the clock (one
  block per 5.333 ms, at most quarter-period nudges, 4 ms of grace for a late
  producer, 7.1 folded to stereo without clipping, each port's initial level
  honoured). The intro movie now plays at −25…−34 dBFS on all six channels.
- **Fog, smoke and the fake water:** the RSX's `WPOS` follows
  `SET_SHADER_WINDOW`, whose origin is bottom-left here; the fragment
  decompiler ignored it, so the ground-fog, smoke and one more shader, which
  look the depth buffer up at 720 − `WPOS.y`, read it upside down, and the
  river level showed a vertically streaked "reflection" over a dry creek bed
  (and over a body and an ammo crate). The window is now applied to `WPOS` in
  the programs that read it, and is part of their pipeline key. Swizzled
  (Morton-order) 16-bit depth shadow maps are written back to guest memory,
  so distant soldiers no longer stand on dark rectangles.
- **The shadow-map pool** (2026-10-24): dark hard-edged rectangles on the
  ground again. The draw engine's colour and depth surface tables (64 slots
  each) never evicted anything, and Resistance packs its shadow maps into a
  per-frame pool whose addresses shift; once the tables were full a new
  shadow pass was dropped without a word and the receivers sampled an empty
  map. The tables now evict the least recently used surface (the log says
  "surface table full: … takes the slot of …").
- **60 fps pacing** (2026-10-24): uncapped, the campaign flips on every
  vblank tick it gets, and on the 62.5 Hz clock a 17 ms frame waited for the
  tick after next (32 ms). With `RES_UNLOCK_FPS=1` the port runs the vblank at
  120 Hz: 88.5 fps in gameplay (it was ~50–56), the game speed unchanged.
  Capping the flips at 60 (`PS3_FLIP_MAX_HZ=60`) measured worse, 50 fps.
  After v0.2.1 (on main) the rate goes to the frame clock inside the process
  (`g_res_vblank_hz`), not through the environment: exitspawn is an `execv`,
  so Save & Quit carried `PS3_VBLANK_HZ=120` back to the menus, whose fixed
  1/60 timesteps could then run at double speed. `PS3_VBLANK_HZ` still
  overrides it.
- **Polygon offset and invariant positions** (2026-10-24): the polygon offset
  the game uses on the character shadow maps, terrain decals and projected
  shadows was ignored; the vertex position is now compiled `invariant`, which
  the foliage's depth pre-pass and `EQUAL` colour pass rely on. Real ZPASS
  occlusion counts are in but off (`PS3RECOMP_ENG_ZPASS=1`): they cost frame
  rate at 60 fps and showed no visible gain.
- **Split-screen** (2026-10-24): `CLEAR_SURFACE` ignored the scissor, so
  player 2's clear wiped player 1's half of the screen. A clear with a scissor
  narrower than the clip now clears only that rectangle (partial-clear passes
  on Metal); one limited only by the clip stays whole.
- **Offline and at quit** (2026-10-24): Multiplayer → Online Game hung
  without a network; the network-start dialog now answers "cancelled" and the
  NP ticket calls exist. The unknown NID called at quit (`0x76948BFC`) is
  `cellSysconfAbort`; with it, `cellSysconfOpen` and the pad sensor-mode
  calls, a full run has no unresolved NIDs.

The toolkit changes are one patch over ps3recomp v0.17.0,
[patches/ps3recomp-a6d76c8-resistance.patch](../patches/ps3recomp-a6d76c8-resistance.patch):
the main-thread stack information, the SPURS kernel context the policy module
reads, the 28-bit vertex offset, the vertex cache and the rest of the
performance work, the fixes from a comparison with RPCS3 (shadow comparison
direction, depth snapshots cropped and released, depth replicated to every
lane, compressed textures never resolved from render targets, PPU threads
starting with VSCR[NJ]=1, no `getenv` on every SPU MFC channel write), the
raw-SPU stop and restart through `SPU_RunCntl` and the masked raw-SPU
fingerprint, cellAdec's AC3 decode and the mixer's pacing, `WPOS` with the
shader window and the swizzled depth write-back, the surface tables' LRU
eviction, the polygon offset, invariant positions, ZPASS occlusion counts
(off by default) and scissored clears, the flip cap, the offline network-start
dialog and NP tickets, `cellSysconfAbort`/`cellSysconfOpen` and the pad
sensor-mode calls, the shared-IOSurface window
(`libs/video/rsx_host_surface.h`, `tools/host_window/host_window.m`; the
host reaps the game with a blocking `waitpid`, so it exits when the game
does, which the launcher's Stop relies on; the title's guest FPS counts each
flip), the window-state hand-off across exitspawn, the GPU depth-as-RGBA8
resolve and the PPU/SPU speed-ups above, what the inFAMOUS 2 work added
since (keyboard and mouse with `input.toml` remapping, rumble, window modes,
vsync and MetalFX in the window host, SMAA with its MIT-licensed shaders in
`third_party/smaa`, the present pass's filters and LUT, texture packs, the
internal resolution), and diagnostics (`VDEC_PICLOG`,
`PPU_STACKINFO_LOG`, `SPURS_PM_R4_WORD0`, `SPURS_PM_TRACE`,
`RSX_ENGINE_PASS_DUMP_TRIGGER`, `RSX_ENGINE_PASS_DUMP_DRAWS`, `RSX_VPC_LOG`,
`PAD_FILE` stick commands, a fuller `AddWorkload` log line).

## Building from source

### The one-click way: the launcher

1. Download the launcher from the
   [releases page](https://github.com/erikomis/resistance-recomp/releases)
   (`.dmg`), drag it to Applications and open it (right-click > Open the
   first time: the builds are unsigned).
2. On **Home**, under **Install the game**, choose your disc image (`.iso`)
   and press **Install the game**.

That is all; everything else happens by itself until **Play** works:

- **Apple's Command Line Tools** (the compiler and `git`): when they are
  missing the launcher opens Apple's own installer (`xcode-select --install`)
  and waits for it. This is the only window you click through.
- **The port's sources**: when this Mac has no checkout, they are cloned into
  `~/Library/Application Support/resistance-recomp/src` (at the launcher's
  version tag when there is one).
- **The build tools and libraries**, without Homebrew:
  [`tools/bootstrap_deps.sh`](../tools/bootstrap_deps.sh) downloads pinned,
  SHA-256-checked versions into
  `~/Library/Application Support/resistance-recomp/deps` (below).
- Then [`tools/install.sh`](../tools/install.sh): the toolkit, a Python
  environment, decrypt, analyse, lift, SPU programs, build, with one progress
  bar and the stage's name.

Measured on a 12-core Apple-silicon Mac with a fast connection: about
7 minutes in all (the build tools 2, the lift and the SPU programs 1, the two
builds 3), and about 1.5 GB of disk (the tools 0.5 GB with their downloads,
the lift 0.25 GB, the builds 0.45 GB); the launcher asks for 4 GB free. Expect
10 to 30 minutes on a smaller Mac or a slower connection. You can keep using
the Mac meanwhile. If you quit, or something fails, pressing the button again
continues where it stopped (finished parts are skipped). A failure says what
went wrong in plain words, with **Copy log**, **Show the log file**
(`~/Library/Application Support/resistance-recomp/install.log`) and **Report a
problem**. **Advanced > Install / rebuild** keeps the step-by-step page for
power users.

### What you need (the terminal way)

- An Apple-silicon Mac (macOS with Metal).
- Your own **Resistance: Fall of Man disc, `BCUS98107` v01.00**, dumped to an
  image and mounted (the folder that holds `PS3_GAME/`).
- An internet connection the first time, to download the decryption keys
  (above); RPCS3 is not needed. The rebuild decrypts the two executables
  itself with `tools/decrypt_self.py`. If that fails and
  [RPCS3](https://rpcs3.net) is installed, it falls back to
  `rpcs3 --decrypt`. ELFs you decrypted some other way work too: drop them
  in as shown below.
- Xcode Command Line Tools (`xcode-select --install`). Everything else comes
  from `tools/bootstrap_deps.sh` (no Homebrew), or, if you prefer, from
  Homebrew:

```sh
brew install cmake ninja sdl2 glslang spirv-cross spirv-tools ffmpeg python@3.12
```

The scripts (`tools/env.sh`) use the `bootstrap_deps.sh` folder whenever it is
there (its `bin/` first on `PATH`; CMake looks for SDL2, glslang and
SPIRV-Cross there and only there; FFmpeg's headers and libraries from it) and
fall back to Homebrew's otherwise.

### Building

```sh
git clone https://github.com/erikomis/resistance-recomp.git
cd resistance-recomp
PS3_VFS_ROOT=/Volumes/<your disc> tools/install.sh   # everything, resumable
```

`tools/install.sh` is what the launcher's button runs: `tools/bootstrap_deps.sh`
(`RES_NO_DEPS=1` skips it and uses Homebrew's), the toolkit, `.venv`, then
`tools/rebuild.sh`'s steps; run it again after a failure and finished steps
are skipped (stamps in `out/install/`). The same by hand:

```sh
tools/bootstrap_deps.sh                       # or the brew install above
tools/setup_toolkit.sh                        # ps3recomp v0.17.0 + patch in ./ps3recomp
"$HOME/Library/Application Support/resistance-recomp/deps/bin/python3.12" -m venv .venv
.venv/bin/pip install -r ps3recomp/requirements.txt
PS3_VFS_ROOT=/Volumes/<your disc> tools/rebuild.sh
```

`tools/bootstrap_deps.sh` (`--list` shows what is installed) puts these into
`RES_DEPS` (default `~/Library/Application Support/resistance-recomp/deps`),
each skipped once it is there, so it can be re-run after a failure:

| Component | Version | How |
| --- | --- | --- |
| CMake | 4.4.4 | Kitware's macOS universal build |
| Ninja | 1.13.2 | ninja-build release |
| Python | 3.12.15 (python-build-standalone 20261009) | `install_only` arm64 build |
| SDL2 | 2.32.10 | from source, shared (the game links it by its absolute path) |
| SPIRV-Headers + SPIRV-Tools | vulkan-sdk-1.4.363.0 | from source, static |
| glslang | vulkan-sdk-1.4.363.0 | from source, static, HLSL + optimizer |
| SPIRV-Cross | vulkan-sdk-1.4.363.0 | from source, static, with the C API |
| FFmpeg | 9.0.2 | from source: LGPL, shared `libavcodec` + `libavutil` only, the H.264, MPEG-2 and AC3 decoders |

Without glslang and SPIRV-Cross the Metal backend cannot translate the game's
shaders, so they are required; FFmpeg is loaded at run time for the movies
and their AC3 sound (`PS3RECOMP_FFMPEG_AVCODEC` / `_AVUTIL`, set by
`tools/env.sh` and the launcher). It takes about 2 minutes on a 12-core Mac
and leaves 0.5 GB.

`tools/rebuild.sh` runs these steps (or only the ones you name, e.g.
`tools/rebuild.sh lift build`):

| Step | What it does | Writes |
| --- | --- | --- |
| `toolkit` | `tools/setup_toolkit.sh` | `ps3recomp/` |
| `decrypt` | decrypts `EBOOT.BIN` and `game.self` from the disc with `tools/decrypt_self.py`, the keys downloaded once by `tools/fetch_keys.py` into `~/Library/Application Support/resistance-recomp/keys/self_keys.txt` (`RES_SELF_KEYS` moves it, `PS3_KEYS` uses a scetool key file you have); falls back to `rpcs3 --decrypt` when that fails and RPCS3 is installed (`RPCS3=<path>`); skipped when the `.elf` is already there; checks the SHA-256 of the result against the tested v01.00 executables | `game/BCUS98107/EBOOT.elf`, `game/BCUS98107/modules/game.elf` |
| `analyse` | the toolkit's `ppu_loader.py` (functions, imports, TOC) | `analysis/BCUS98107/` |
| `lift` | the toolkit's `ppu_lifter.py`, 16 chunks for the EBOOT and 12 for `game.self`, with the switchable instructions of the 60 fps patches (`port/patches/*.insn.json`) | `recompiled/BCUS98107/`, `recompiled/BCUS98107/game_module/` |
| `spu` | `tools/regen_spu_pm.sh`: the job-manager policy module, extracted from your EBOOT, wrapped and lifted; `tools/regen_jobmods.sh`: the 15 WWS job modules; `tools/regen_rawspu.sh`: the two raw-SPU programs, cut out of `game.elf` | `port/spu/captured/`, `port/spu/jobmod_*/`, `port/spu/rawspu0/`, `port/spu/rawspu1/` |
| `build` | CMake + Ninja, Metal, FFmpeg runtime; also the `Resistance` window launcher | `build/BCUS98107/ResistanceRecomp`, `build/BCUS98107_game/ResistanceRecomp`, `build/BCUS98107/Resistance` |
| `v17` | only when named: the v1.7 campaign from your `patch_06.psarc` (`tools/v17/regen_v17.sh`, see [The v1.7 campaign](#the-v17-campaign)); `build` also rebuilds it once it is lifted | `game/BCUS98107/modules_v17/`, `recompiled/BCUS98107/game_module_v17/`, `port/spu_v17/`, `build/BCUS98107_game_v17/` |
| `launcher` | only when named: the desktop [launcher](#launcher) app (`npm ci`, `tauri build`; Node 20+ and Rust) | `build/launcher/Resistance Launcher.app` |

Everything in that last column is generated from your copy and is
`.gitignore`d. Keep the layout: when the EBOOT hands off to the campaign,
`port/stubs.cpp` runs `build/BCUS98107_game/ResistanceRecomp` with
`game/BCUS98107/modules/game.elf` next to the EBOOT build, or
`build/BCUS98107_game_v17/ResistanceRecomp` with
`game/BCUS98107/modules_v17/game.elf` when both exist (`RES_GAME_VERSION=100`
keeps 01.00; `RES_SPAWN_BIN` / `RES_SPAWN_ELF` override all of that).

The executable was called `MyGameRecomp` until 2026-10-22, and builds made
before then were compiled unoptimised: rebuild. `tools/setup_toolkit.sh` never
resets a checkout that carries an older patch, so delete `ps3recomp/` first,
then run `tools/rebuild.sh toolkit build`. Until you do, the scripts still run
an old `MyGameRecomp` build when it is the only one there.

A lift made before 2026-10-23 has no 60 fps patch in it (`RES_UNLOCK_FPS` and
`RES_MENU_60FPS` then do nothing): after updating the toolkit, run
`tools/relift.sh` (it re-lifts the EBOOT, the 01.00 and the v1.7 campaign with
`--insn-patches` and rebuilds; only one or two chunks change). A lift made
before 2026-10-25 also lacks Lua's `setjmp`/`longjmp` (`--sjlj`, without it
Campaign → Location Select quits the game): after `git pull`, delete
`ps3recomp/`, run `tools/rebuild.sh toolkit` and then `tools/relift.sh`, and
the Grimsby freeze fix comes with the same toolkit update.

The Continue, movie-sound and fog fixes are in the toolkit patch and in the
SPU registration: after `git pull`, delete `ps3recomp/` and run
`tools/rebuild.sh toolkit spu build`, plus `tools/v17/regen_v17.sh spu build`
for the v1.7 campaign (it copies the masked registration into
`port/spu_v17/`). The raw-SPU capture run (`SPU_RAW_DUMP_DIR`) is no longer
needed.

v0.2.1 (2026-10-24): the shadow-pool, split-screen, polygon-offset and
offline fixes are in the toolkit patch and the 120 Hz vblank is in
`port/patches.cpp`. Update an existing checkout the same way: after
`git pull`, delete `ps3recomp/` and run `tools/rebuild.sh toolkit spu build`,
plus `tools/v17/regen_v17.sh spu build` for the v1.7 campaign.

After v0.2.1 (on main, in the next release): the fix that keeps the 120 Hz
vblank out of the environment is in the port only (`port/main.cpp`,
`port/patches.cpp`); after `git pull`, `tools/rebuild.sh build` is enough.

#### The SPU policy module

The job-manager policy module is translated game code, so it is regenerated
locally, never committed. It is the 11,136-byte blob the title passes to
`cellSpursAddWorkload` (guest 0x10191A80 in the EBOOT, 0x10176E80 in
`game.self`), fingerprint `4E4076E700C6AFF3`. `tools/regen_spu_pm.sh` does:

```sh
tools/extract_pm.py game/BCUS98107/EBOOT.elf -o out/spu_pm/pm.bin     # checks the fingerprint
python ps3recomp/tools/wrap_spu_elf.py out/spu_pm/pm.bin --base 0xA00 --entry 0xA00 --out out/spu_pm/pm.elf
python ps3recomp/tools/spu_lifter.py --auto-functions out/spu_pm/pm.elf --base 0xA00 \
       --symbol-prefix resistance_spu --output port/spu/captured
```

The same blob can be captured at runtime instead: run without a lifted PM and
with `SPU_DUMP_MISS=<dir>`, and pass the resulting
`spujob_4E4076E700C6AFF3_11136.bin` to `tools/regen_spu_pm.sh`.

#### The raw-SPU programs

When a level starts, `game.self` streams two programs into raw SPUs by proxy
DMA (local store 0x60 onward, entry 0x90). Both are linked into its data
segment, and its PPU loader is only ever handed those two (constant
{pointer, size} pairs in the GOT), so no level loads anything else.
`tools/extract_rawspu_elf.py` cuts them out of the ELF and they are lifted at
base 0x60 (the `spu` step of `tools/rebuild.sh` does this):

```sh
tools/regen_rawspu.sh game/BCUS98107/modules/game.elf   # -> port/spu/rawspu0, port/spu/rawspu1
```

`port/spu/spu_workloads.c` registers them as SPU images 2
(`1894F66385AFD052`) and 3 (`2B2DB6A5DE233A5F`). The second carries a
0x50-byte parameter area that the PPU fills in before every load (heap
pointers, per-level values, its raw SPU's MMIO window), so the bytes it is
loaded with, and their fingerprint, change with the level, the checkpoint and
the SPU number. It is registered with that area masked
(`spu_workload_register_img_masked`): the fingerprint is that of the program
with the area zero, as it is in the ELF. Like the policy module, the lifted C
is translated game code and stays out of git.
`port/spu/spu_workloads.c` registers the lifted entry
(`resistance_spu_spu_func_00000A00`) under that fingerprint as SPU image 1.

#### The v1.7 campaign

The game's last update, v1.7, is seven archives, `patch_01`…`patch_07.psarc`;
`patch_06` carries a new campaign executable (`/data/new_game.self`). The
port runs it. Its decrypted ELF (SHA-256 `e1d75e6d4afe…144f`, RPCS3's
"game.self Updated", PPU-a08a187d…) is lifted like the disc's (19,706
functions against 19,219), and 13 of the 15 WWS job modules and both raw-SPU
programs changed and are lifted from it (new fingerprints `6D03497950639952`
and `F9726FDA4A599084`, the latter with its parameter area masked as for
01.00; the policy module is byte-identical). It imports
`cellSaveDataAutoSave2`/`AutoLoad2`, `_sys_process_atexitspawn` and
`cellPadSetActDirect` (rumble), which 01.00 does not.

The patch files are the game's official free updates and are not in this
repository. If you own the game, [PSRewired](https://psrewired.com) lets it
download its own update in RPCS3; copy `patch_01`…`patch_07.psarc` from
RPCS3's `dev_hdd0/game/BCUS98107/USRDIR/data` to `update/` in this checkout
(or point `RES_UPDATE_DIR` at them), then:

```sh
tools/rebuild.sh v17                                  # tools/v17/regen_v17.sh
```

`regen_v17.sh` extracts `new_game.self` from `patch_06`
(`tools/v17/psarc_extract.py`), decrypts it as the `decrypt` step does into
`game/BCUS98107/modules_v17/game.elf` (checking the SHA-256), lifts it to
`recompiled/BCUS98107/game_module_v17/`, lifts its SPU code into
`port/spu_v17/` (the policy module is copied from `port/spu/captured/`, so
run the `spu` step first; the raw-SPU programs are cut out of the v1.7 ELF
as for 01.00) and builds `build/BCUS98107_game_v17/`.

The menus stay 01.00. The update's `EBOOT.BIN` is only available in NPDRM
(digital-edition) form, and the disc's 01.00 EBOOT cannot run with the
update's data mounted (with `patch_04` or `patch_06` it crashes loading the
menu fonts; with the others it hangs at the exitspawn). So **do not install
the patch files in the play folder**: `tools/play.sh` and the launcher refuse
to start while a `patch_NN.bin` is in its `USRDIR/data`. The campaign does
not need them: they hold front-end and multiplayer data, and `game.self`
never mounts the patch archives. At the exitspawn the log says
`campaign = v1.7 game.self`; `RES_GAME_VERSION=100` plays the 01.00 campaign.

## Playing

Mount your disc image, plug in or pair a controller (DualSense, DualShock or
Xbox) or use the keyboard and mouse, then:

```sh
tools/play.sh
```

It starts the front end in a window owned by the `Resistance` launcher
(`play.sh` builds it in seconds if a build from before it lacks it); starting
a campaign hands off to the `game.self` build in the same window. `play.sh`
says which campaign that is: v1.7 when it is built (and it builds it when the
v1.7 code is lifted but not compiled), else 01.00 (see
[The v1.7 campaign](#the-v17-campaign)). It refuses to start while the v1.7
update's `patch_NN.bin` files are in the play folder. On the **first boot press START
twice**: on the legal screen, then on the "initial setup" prompt (the HDD
install, done once). Saves, the install cache and the log (`last.log`) are
kept in `~/Library/Application Support/resistance-recomp/` (`RES_DATA`
changes that). The window title shows the frame rate. Close the window, or
press Ctrl+C in the terminal, to quit. If a build is missing, `play.sh` says
so: run `tools/rebuild.sh`.

Keyboard and mouse (new, not yet play-tested; a click in the window captures
the mouse, Esc releases it):

| Action | Keys and mouse | PS3 |
| --- | --- | --- |
| Move / look | W A S D / the mouse (or I J K L) | left / right stick |
| Fire / aim | left click or X / right click or Z | R1 / L1 |
| Grenade | G or mouse button 4 | L2 |
| R2 | middle click or 1 | R2 |
| Jump / crouch | Space / C, Ctrl or Backspace | ✕ / ○ |
| Reload, use | R, E or mouse button 5 | □ |
| Switch weapon | Q or the wheel | △ |
| L3 / R3 | Shift / V | L3 / R3 |
| Start / select | Return, Esc or P / Tab or M | START / SELECT |

Rebind them on the launcher's Controls page, or in an `input.toml` passed as
`PS3_INPUT_MAP` (the format is in the toolkit's `libs/input/pad_input_map.h`).

## In-game menu

Press **F1** (fn+F1 on a Mac keyboard, where F1 is the brightness key),
**Insert** (Help on older Mac keyboards) or **Cmd+,** to open and close it. On a
controller, press **L3 and R3** (both sticks) together. Esc or ○ also closes it.
L1/R1 switch tabs, the D-pad or left stick and ✕ work the controls, and the
mouse clicks. While the menu is open the game gets no input: its pad, keys and
mouse are held neutral and the cursor is released. The game keeps running, so
pause it first with START if you need to. A key, click or button that is still
held when the menu closes (the Esc that closed it, for example) only reaches
the game after you let go of it. One stick click alone still reaches the game.
While both sticks are held down, neither click does.

The menu works in the game's single window (`Resistance`, `tools/host_window`).
The window counts the menu's keys and passes the pointer to the game. The game
draws the menu into the picture it hands the window. It also works when
`ResistanceRecomp` runs with its own window. It works across the switch from
the front end (EBOOT) to the campaign (game.self) as well: both executables load
the same settings, and anything changed in the front end carries over into the
campaign.

**Quit game** is at the bottom, next to Close. It asks first ("Progress since
the last checkpoint will be lost"). Cancel has the focus, and Esc or ○ backs
out. Quit ends the game the same way closing the window does, but it never
stops the game in the middle of a save. As soon as you quit, the game starts no
new save, profile write or checkpoint write (`quick.sav`). One that is already
being written finishes first, for up to 30 s. Then the game exits with status 0,
and the window closes with it.

| Tab | What it changes |
| --- | --- |
| Image | Changes at once: gamma override (or the game's own brightness), sharpening, vibrance, contrast, vignette, film grain, LUT strength (when `PS3_LUT` loaded one), anti-aliasing off/FXAA/SMAA, MetalFX upscaling, vsync, and window / macOS fullscreen / borderless. Changes at the next start: anisotropic filtering and internal resolution (720p, 1080p, 1440p, 4K). A change made in the front end also applies when the campaign loads. |
| Audio | Master volume (what the Mac plays; the game's own mix is untouched) |
| Controls | Play with controller + keyboard/mouse, controller only, or keyboard/mouse only. Vibration on/off and strength. Mouse camera on/off, sensitivity, and vertical/horizontal inversion. A table of every PS3 button with its key, mouse button and controller button: click a cell (or pick it with the pad and press ✕), then press the new input. It applies at once. Esc cancels, right-click unbinds, and "Restore defaults" resets the table. The press goes to the menu, never to the game. Rebindings are saved straight away to the controls file (`PS3_INPUT_MAP`, or `input.toml` next to `overlay.ini`). |
| Performance | A small HUD in a corner that stays up when the menu is closed. It shows fps, frame time with a graph, CPU cores in use, busy threads (≥ 80% of a core, mostly the SPU threads and the PPU), the RSX's Metal encode time and the wait for a frame slot. You can set its corner, opacity and graph. |

MetalFX, vsync and the display mode belong to the window. The menu asks the
window to change them, so they work in the game's window (`Resistance`). With
its own window, `ResistanceRecomp` does vsync and macOS fullscreen but not
MetalFX.

**Where the settings go.** The menu saves its changes when it closes, to the
file named by `PS3_OVERLAY_INI`. `jogar.sh` points it at
`~/Library/Application Support/resistance-recomp/overlay.ini`. If you leave the
variable unset or empty, changes last for the session only. The file holds only
what you changed in the menu, as lines with the same names and values as the
variables (`PS3_SHARPEN=0.40`, `PS3_AUDIO_VOLUME=70`, `PS3_OVERLAY_HUD=1`, ...).
The window reads `PS3_VSYNC`, `PS3_WINDOW_MODE` and `PS3_UPSCALE` from the same
file. At start, the first of these wins:

1. a variable set when the game starts (from the shell, or from the launcher,
   which should pass only the settings it has away from their defaults);
2. the menu's line in `overlay.ini`;
3. the built-in default (and, for the mouse, `[mouse]` in the controls file).

The menu marks a setting whose variable was set with an orange "env". A change
to it applies at once, but the variable wins again the next time. A small
notice in the corner says when the settings were saved.

| Variable | What it does |
| --- | --- |
| `PS3_OVERLAY=0` | no menu and no HUD (nothing is loaded) |
| `PS3_OVERLAY_INI=<file>` | where the menu's settings are read from and saved to (unset or empty = only for the session) |
| `PS3_OVERLAY_PAD=0` | L3+R3 does not open the menu |
| `PS3_OVERLAY_HUD=1` | the performance HUD on from the start; `PS3_OVERLAY_HUD_CORNER=0..3` (top left, top right, bottom left, bottom right), `PS3_OVERLAY_HUD_OPACITY=0.1..1`, `PS3_OVERLAY_HUD_GRAPH=0` |
| `PS3_OVERLAY_LANG=en\|pt` | the menu's language (default: Portuguese when `PS3_LANG` starts with `pt`, else English) |
| `PS3_INPUT_MAP=<file>` | the controls file the menu's rebindings go to (unset: `input.toml` next to `PS3_OVERLAY_INI`, which is then read at the next start) |

**Testing it headless** (`PS3RECOMP_METAL_HEADLESS=1`). A headless run reads no
settings file and draws nothing unless a test asks:

| Variable | What it does |
| --- | --- |
| `PS3_OVERLAY_FORCE_OPEN=1` | the menu drawn open (`=quit`: on the "Quit the game?" question); the frame dump (`PS3RECOMP_METAL_FRAME_DUMP`) then reads the composed picture |
| `PS3_OVERLAY_TAB=0..3` | the tab it opens on: Image, Audio, Controls, Performance |
| `PS3_OVERLAY_TEST="<s>:<action>,..."` | a timed script, in seconds from the backend's start: `open`, `close`, `tab=<n>`, `hud=<0\|1>`, `set=<VARIABLE>:<value>` (as the menu sets it), `shot=<file.ppm>` (writes the next composed frame), `quit` (opens the question), `confirm` (presses Quit: the real quit path), `cancel` |
| `PS3_OVERLAY_TEST_NEXT=...` | the script for the executable the exitspawn brings up (each one takes its own) |

For example, this checks every tab and the quit, and the run ends with status 0:

```sh
PS3RECOMP_METAL_HEADLESS=1 PS3_OVERLAY_TEST="25:tab=0,28:shot=/tmp/t0.ppm,31:tab=1,34:shot=/tmp/t1.ppm,\
37:tab=2,40:shot=/tmp/t2.ppm,43:tab=3,46:shot=/tmp/t3.ppm,49:close,60:quit,66:confirm" \
  build/BCUS98107/ResistanceRecomp game/BCUS98107/EBOOT.elf
```

Not in this menu: the inFAMOUS 2 port's Game tab (game languages; Resistance
reads `PS3_LANG` at start), button-icon textures, the aspect ratio
(ultrawide), colour curves, clarity and colour presets. This toolkit has none
of these. The menu comes from the inFAMOUS 2 port (Dear ImGui,
`third_party/imgui`, MIT).

## Launcher

[`launcher/`](../launcher/) is a desktop app (Tauri) that does what
`tools/play.sh` does without a terminal, modelled on the inFAMOUS 2 port's
launcher: it finds your builds and your disc, mounts the disc image for you
(`hdiutil`, read-only) and remembers it, keeps the settings, and **Play**
starts the `Resistance` window host with the EBOOT build, the play folder
(`PS3_HDD0_ROOT`/`PS3_HDD1_ROOT`, `last.log`), FFmpeg and `caffeinate`, as
`play.sh` does. **Stop** ends the game and its window (the window host now
exits when the game does). It ships nothing from the game: the cover on its home screen
is read from your mounted disc.

- **Install the game** (Home, until the game is built): choose the disc
  image and press one button; the launcher gets Apple's Command Line Tools
  (Apple's installer), the sources, the build tools (no Homebrew, no RPCS3)
  and builds the game with one progress bar, resumable
  ([Building from source](#building-from-source)).
- **Home**: Play/Stop, what still blocks Play (no build, no disc, the v1.7
  update's `patch_*.bin` in the play folder, which the 01.00 EBOOT cannot run
  with; one click moves them aside), the first-boot hint (two STARTs), quick
  settings, the last save, status, a Play shortcut on the Desktop and Add to
  Steam.
- **Graphics**: display mode (window, macOS fullscreen, borderless), window
  size (the host's `--size`), screen, vsync, the Dock icon from your disc;
  internal resolution (experimental), MetalFX upscaling, FXAA or SMAA,
  sharpening, anisotropic filtering, a gamma override, profiles; filters
  (vibrance, contrast, vignette, grain, a colour LUT file); texture packs
  (enable and order the packs in the mods folder, dump the game's textures).
- **Performance**: the vertex cache and its per-stream split, binding only
  changed state, the shader cache (with its size and Clear) and async
  shaders, the frame clock, the frame-rate counter, keeping the Mac awake;
  **60 fps** (`RES_UNLOCK_FPS` + `RES_MENU_60FPS`; the port then runs the
  vblank at 120 Hz) and the field of view
  (`RES_FOV`), shown once the build has them.
- **Game**: the disc (image or folder, mount/eject), the campaign version
  (**v1.7** when its build is there, or **01.00**: `RES_GAME_VERSION=100`),
  the console language (`PS3_LANG`), movies (FFmpeg paths), audio (volume,
  mute, output device), the play folder, and a saves manager (the profile and `quick.sav`, here and in RPCS3:
  backups before every Play, restore, copy either way, .zip export/import).
- **Controls**: the bindings for keyboard, mouse and controller with what
  each button does in the game (written to `input.toml`), a controller test,
  what drives the game (both, controller only, keyboard and mouse only), mouse
  sensitivity, inversion, dead zone and capture, vibration and its strength.
  Every setting reaches the game only when it differs from the default.
- **Advanced**: Install / rebuild (the requirements, with "Install the build
  tools" = `tools/bootstrap_deps.sh`, the port folder, `tools/rebuild.sh` with
  chosen steps and a progress bar),
  Updates (`git fetch` of the checkout, "builds older than the sources", the
  newest launcher release), the game log viewer, debug mode (a session folder
  with captures, a log summary and a .zip to attach to an issue), and the raw
  environment with the equivalent command line.

UI in Brazilian Portuguese and English. Build it with Node 20+ and Rust:

```sh
tools/rebuild.sh launcher        # -> build/launcher/Resistance Launcher.app
```

Details, screenshots and the development commands are in
[launcher/README.md](../launcher/README.md).

## Playing from the terminal

Mount your disc, then:

```sh
tools/start.sh menu                  # the EBOOT in a window, in the background
tools/press.sh menu legal 0x0008     # START on the legal screen
tools/press.sh menu setup 0x0008     # START on the first-run install prompt
tools/snap.sh menu now               # out/menu/now.png
tools/stop.sh
```

The first boot needs **two START presses**: the legal screen, then the
"initial setup" prompt (the HDD install, done once; `start.sh` keeps its run
directory, `out/game.cwd/`, between runs). With the install done,
`tools/to_campaign.sh <name>` drives the menu into a new Medium campaign and
the hand-off to `game.self`.

| Script | What it does |
| --- | --- |
| `tools/play.sh` | play in a window with a controller (see [Playing](#playing)) |
| `tools/run.sh <name> <seconds> [eboot\|game]` | timed run, a PNG of the frame every 10 s in `out/<name>/`, log in `out/<name>.log`; fresh run directory unless `KEEP=1` or `CWD=<dir>` |
| `tools/start.sh <name> [eboot\|game]` | background run driven through `PAD_FILE`; persistent `out/game.cwd/` |
| `tools/press.sh <name> <tag> <mask> [polls] [wait]` | press a button, wait, save the frame as `<tag>.png` |
| `tools/snap.sh <name> <tag>` | save the current frame |
| `tools/stop.sh` | stop the `start.sh` run (only that one) |
| `tools/to_campaign.sh <name>` | menu → new Medium campaign → `game.self` |
| `tools/ppm2png.py in.ppm out.png` | frame dump to PNG, no dependencies |

Button masks: START `0x0008`, UP `0x0010`, RIGHT `0x0020`, DOWN `0x0040`,
LEFT `0x0080`, TRIANGLE `0x1000`, CIRCLE `0x2000`, CROSS `0x4000` (OR them
for chords).

### Environment

The scripts set these for you ([tools/env.sh](../tools/env.sh)); to run a build
directly: `build/BCUS98107/Resistance -- build/BCUS98107/ResistanceRecomp
game/BCUS98107/EBOOT.elf` (or without the launcher, `ResistanceRecomp` opens
its own window).

| Variable | What it does |
| --- | --- |
| `PS3_VFS_ROOT=<dir>` | the mounted disc, the folder holding `PS3_GAME/` (the scripts find a mounted `BCUS98107` under `/Volumes` when unset) |
| `PS3_HDD0_ROOT`, `PS3_HDD1_ROOT` | writable folders for `/dev_hdd0` (install cache, saves, profile) and `/dev_hdd1` |
| `PS3RECOMP_FFMPEG_AVCODEC`, `PS3RECOMP_FFMPEG_AVUTIL` | paths of `libavcodec` / `libavutil` (on macOS, e.g. `/opt/homebrew/lib/libavcodec.dylib`); `PS3RECOMP_FFMPEG=0` turns video decoding off |
| `PAD_SCRIPT="<sec>:<mask>,..."` | timed presses, e.g. `PAD_SCRIPT="40:0x0008,70:0x0008"` for the two first-boot STARTs |
| `PAD_FILE=<file>` | closed-loop input: write `<mask> [polls]` to the file and the pad presses it on its next poll, then empties it; `S <lx> <ly> <rx> <ry> <polls>` holds the sticks instead (0–255, 128 = centre) |
| `PS3RECOMP_METAL_FRAME_DUMP=<file.ppm>` | rewrites that file with the frame every 120 presents |
| `PS3RECOMP_METAL_HEADLESS=1` | no window (off-screen rendering) |
| `PS3RECOMP_METAL_FPS=1` | frame rate and frame times in the window title (and `[RSX FPS]` log lines) |
| `RSX_ENGINE_PASS_DUMP_TRIGGER=<file>` | with the draw engine's per-pass dump (`RSX_ENGINE_PASS_DUMP=<dir>`), start capturing when that file appears, so a script can dump the frame it has steered the game to |
| `RSX_ENGINE_PASS_DUMP_DRAWS=1` | with the per-pass dump, one line per draw in `draws.txt`, now with each vertex attribute's format and first bytes |
| `RSX_VPC_LOG=<constant>[,<draw>]` | every 30 frames, log a vertex-program constant as it stood at that draw (default draw 40) with the frame's draw count; `c16` is the camera position in Resistance's world shaders |
| `PS3RECOMP_ENG_VCACHE=0`, `PS3RECOMP_ENG_VCACHE_SPLIT=0` | turn off the cross-frame vertex cache / its per-stream split (static mesh streams cached apart from the per-frame lighting stream) |
| `PS3RECOMP_METAL_BIND_ALL=1` | rebind every Metal state on every draw instead of only what changed |
| `PS3_SHADER_CACHE=<dir>` | keep the translated MSL shaders across runs; `PS3_SHADER_ASYNC=1` (opt-in) builds new pipelines on a background queue and skips their draws until ready |
| `PS3_FRAMECLOCK_US=<µs>` | the host frame loop's tick (default 1000; 4000 is the old pacing) |
| `PS3_VBLANK_HZ=<10..1000>` | the synthesized vblank rate (default 62.5; with `RES_UNLOCK_FPS=1` the campaign uses 120 unless this is set, without putting it in the environment, so the menus after Save & Quit keep 62.5) |
| `PS3_FLIP_MAX_HZ=<10..1000>` | complete flips no closer together than 1/hz (off by default; capping the unlocked campaign at 60 measured worse, 50 fps) |
| `PS3RECOMP_ENG_POLY_OFFSET=0`, `PS3RECOMP_MSL_INVARIANT=0`, `PS3RECOMP_ENG_RECT_CLEAR=0` | turn off the polygon offset / the invariant vertex position / scissored clears (each is on by default) |
| `PS3RECOMP_ENG_ZPASS=1` | real ZPASS occlusion counts (off by default: they cost frame rate at 60 fps) |
| `RES_HANG_SECS=<s>` | the hang report: after that many seconds without a new frame (default 30; `0` turns it off), once per stall, every guest thread's last direct call, the threads inside an HLE call, every SPU context (pc, events, parked DMA list, mailboxes, reservation) and a 3-second host `sample` go to the log (`[hang]` lines); nothing is stopped |
| `PS3_NET_ONLINE=1`, `PS3_NP_SIGNIN=1`, `PS3_NP_ONLINE_ID=<name>`, `PS3_NET_DNS=<ip>[:port]`, `PS3_NET_HOSTS=name=ip,...` | online experiments, all off by default ([plan](../ROADMAP.md#online-multiplayer-through-ps-rewired-future)): real sockets and a connected network; signed in to PSN with a locally made ticket for that online ID; the DNS the game is told about and the runtime's lookups use (a port other than 53 for local tests); fixed answers for names (`*.suffix`, `*`). Guards: `PS3_NET_ALLOW=ip,...` (the only addresses the game may reach, loopback aside), `PS3_NET_LOCAL_IP=<ip>` (the LAN address the game is told instead of this computer's), `PS3_NET_NO_UPNP=1` (UPnP discovery dropped). `PS3_NET_TRACE=1` logs every socket call |
| `PS3_MIC=1`, `PS3_MIC_TONE=1` | voice chat: the computer's default microphone as the PS3 headset (the app passes it to Play online when Voice chat is on), or a 440 Hz test tone that records nothing |
| `RES_LEVEL=<n>` | start the campaign in level `n` (the disc's `packed/levels/level<n>`: 20–22 York, 30–32 Grimsby, 40–42 Manchester, 50–52 Nottingham, 60–62 Cheshire, 70–72 Somerset, 80–82 Bristol, 90–91 Bracknell, 100–101 and 110–112 London, 120–121 the Chimeran Tower): set it, start a new campaign, and `port/main.cpp` writes the level into the exitspawn block. For checks and bug repros |
| `RES_UNLOCK_FPS=1`, `RES_MENU_60FPS=1`, `RES_FOV=<0.1..4>` | the optional game patches ([`port/patches.cpp`](../port/patches.cpp)): the campaign's frame limiter off (minimum timestep 1/30 → 1/240 s), the menus and multiplayer at one vblank per frame, the field-of-view factor (1.0 = the game's; smaller is wider). Each applies only when the running ELF holds the words it expects |
| `PS3_AUDIO_VOLUME=0..100`, `PS3_AUDIO_DEVICE=<name>`, `PS3_AUDIO_MUTE=1` | master volume (100), the output whose name contains `<name>` (the outputs are listed in the log when none matches; macOS/Linux), no sound (any value mutes) |
| `PS3_AUDIO_DUMP=<file.wav>`, `PS3_AUDIO_STATS=1` | record the final mix (48 kHz stereo float; before mute and volume; the menus and the campaign go into one file) / every ~5 s, log each audio port's and the mix's level, silent blocks and clipped samples |
| `PS3_LICENSE_AREA=<n>` | the licence area the console reports (`CELL_SYSUTIL_LICENSE_AREA_*`; default North America, the disc's region) |
| `PS3_USLEEP_SPIN_US=<µs>` | a `sys_timer_usleep` of that many µs or less spins (yielding) instead of sleeping (opt-in; spinning threads cost cores the SPUs may need) |
| `PS3_INPUT_MAP=<file>` | keyboard, mouse-button and controller bindings and the `[mouse]` settings (`input.toml`; format in the toolkit's `libs/input/pad_input_map.h`) |
| `PS3_INPUT_MODE=both\|controller\|keyboard` | what drives the game (default both); `PS3_PAD_SDL=0` ignores physical controllers |
| `PS3_MOUSE=0`, `PS3_MOUSE_SENS=<f>`, `PS3_MOUSE_INVERT_X=1`, `PS3_MOUSE_INVERT_Y=1`, `PS3_MOUSE_DEADZONE=0..120`, `PS3_MOUSE_CAPTURE=0` | the mouse camera: off, speed (1.0), inversion, first stick step past centre (30), no cursor capture |
| `PS3_CLEAN_EXIT=0` | the old quit: the window closing or SIGTERM no longer ends the process (by default it waits for a save in progress, at most 2 s, and exits) |
| `RES_SPEAKERS=0` | the game's speaker-count getter reads its global again (the race that sometimes mixed a session in the 2ch layout) |
| `PS3RECOMP_FP_ALPHA_QUANT=0` | the alpha test compares the raw alpha (the old behaviour; the grass layer then disappears) |
| `PPU_STWCX_SAME_NOTIFY=1`, `SPU_WAKE_LOCK=0`, `SPU_WAIT_SPIN_US=<µs>` (0 = off, default 100), `SPURS_RCS_FAST=0`, `SPU_BRANCH_ALIGN=0` | turn off, one by one, the SPU and SPURS changes from the Resistance PC port (a same-value `stwcx.` keeping the SPUs' reservations, the locked channel wake, the raw SPUs' short spin before sleeping, the lock-free `cellSpursReadyCountStore`, word-aligned SPU indirect branches); `-DRES_SPU_FAST=OFF` at configure brings back the local-store debug probes (`SPU_LS_WATCH` and the like) |
| `PS3_PAD_LOG=1` | logs every change of the physical buttons and of the packets the game reads (`[cellPad] physical …` / `packet …`), to check a controller |
| `PS3_RUMBLE=0`, `PS3_RUMBLE_STRENGTH=0..200` | vibration off / its strength in percent (100) |
| `PS3_WINDOW_MODE=windowed\|fullscreen\|borderless`, `PS3_WINDOW_SIZE=WxH`, `PS3_DISPLAY=<n\|name>`, `PS3_VSYNC=0`, `PS3_UPSCALE=metalfx`, `PS3_DOWNSCALE=0` | the window host: how the window opens (letterboxed; `--size` still works), on which screen, vsync off, MetalFX upscaling; a picture larger than the window (internal resolution above the screen's) is reduced into it with a Lanczos filter, supersampling, unless `PS3_DOWNSCALE=0` |
| `PS3_AA=fxaa\|smaa`, `PS3_SHARPEN`, `PS3_ANISO`, `PS3_DISPLAY_GAMMA` | anti-aliasing, sharpening (0–1), anisotropic filtering, display gamma |
| `PS3_VIBRANCE=-1..1`, `PS3_CONTRAST=0.5..1.5`, `PS3_VIGNETTE=0..1`, `PS3_GRAIN=0..1`, `PS3_LUT=<png>`, `PS3_LUT_STRENGTH=0..1` | the present pass's filters and a colour LUT (ReShade's PNG layout) |
| `PS3_MODS=<pack,...>`, `PS3_MODS_DIR=<dir>`, `PS3_MODS_DUMP=<dir>` | texture packs (folders of `<key>.png`, first wins) and a dump of every texture under its key |
| `PS3_RES_SCALE=1\|1.5\|2\|3\|4` (or `720p`, `1080p`, `1440p`, `4k`) | internal resolution (experimental) |
| `SPURS_PM_TRACE=1` | log the job manager's policy-module polls (`[spurs-pm]`; off by default, it costs frame rate) |
| `PS3_TITLE` | window title |
| `RES_GAME_VERSION=100` | the 01.00 campaign even when the v1.7 build is there |
| `RES_UPDATE_DIR=<dir>` | where `tools/v17/regen_v17.sh` finds `patch_06.psarc` (default `update/`) |
| `RES_SPAWN_BIN`, `RES_SPAWN_ELF` | the build and ELF the exitspawn hand-off runs (default: the sibling build directory and `game/BCUS98107/…`) |
| `PS3RECOMP_HOST_SURFACES` | set by the `Resistance` launcher: the Mach bootstrap name of the shared window surfaces; not meant to be set by hand |
| `PS3RECOMP_WINDOW_STATE` | set by the port across exitspawn so the next window opens where the last one was; not meant to be set by hand |
| `RES_SPAWN_KEEP=1` | keep the exitspawn data file (to rerun `game.self` by hand with `RES_SPAWN_DATA=<file>`) |
| `RES_PEEK=<secs>:<addr>[,*<addr>...]` | every `<secs>`, print 32 bytes of guest memory at each address (`*` dereferences first) |
| `SPU_DUMP_MISS=<dir>` | write every SPU image/policy module that has no lifted code to `<dir>` |
| `VDEC_PICLOG=1`, `PPU_STACKINFO_LOG=1` | log the first decoded pictures / every stack-information query |

## Layout

| Path | What it is |
| --- | --- |
| [`port/`](../port/) | The port: `main.cpp` (boot, guest thread, exitspawn data block), `stubs.cpp` (exitspawn and the window-state hand-off, `RES_PEEK`), `patches.cpp` and `patches/*.insn.json` (the optional 60 fps / FOV patches), `CMakeLists.txt` (with the `Resistance` launcher target), `config.toml`, `spu/spu_workloads.c` (SPU registration); `spu_v17/` is generated by `tools/v17/regen_v17.sh` |
| [`patches/`](../patches/) | The toolkit patch over ps3recomp v0.17.0 |
| [`tools/`](../tools/) | `setup_toolkit.sh`, `rebuild.sh`, `relift.sh`, `regen_spu_pm.sh`, `extract_pm.py`, `regen_rawspu.sh`, `extract_rawspu_elf.py`, `play.sh` and the run scripts; `v17/` (the v1.7 campaign: `regen_v17.sh`, `psarc_extract.py`, `spu_workloads.c`) |
| [`launcher/`](../launcher/) | The desktop launcher (Tauri): settings, disc, saves, rebuild and Play |
| [`ROADMAP.md`](../ROADMAP.md) | Status and next steps |

## License

Code in this repository: MIT ([LICENSE](../LICENSE)). The toolkit is MIT by its
authors. Resistance: Fall of Man © Sony Interactive Entertainment LLC,
developed by Insomniac Games; this project is not affiliated with Sony or
Insomniac.
