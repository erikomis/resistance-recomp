# Resistance: Fall of Man — static recompilation for PC (macOS)

🌐 **English** · [Português (BR)](README.pt-BR.md)

A native port of **Resistance: Fall of Man** (PS3, disc `BCUS98107`, v01.00)
to **macOS/arm64**, built with static recompilation: the game's PowerPC and
SPU code is translated to C/C++ and compiled into native executables, with
graphics on Metal. It is not an emulator.

**It reaches gameplay, and chapter 1 has been played with a controller.**
The front end runs from the legal screens through the first-run install, the
title screen and the main menu to a new campaign. The campaign executable then
plays the level-1 intro movie and loads "The Gauntlet" (York). In a window,
with a DualSense, York plays in first person with the HUD: enemies and allies,
shooting, dying and respawning from the checkpoint. The town renders with its
proper colours at about 30 fps, in one window from the menus through the
campaign. The campaign is the game's last update, v1.7, re-lifted from its
`patch_06`; the menus stay the disc's 01.00 (see [Status](#status)).

Built on [ps3recomp](https://github.com/sp00nznet/ps3recomp), following the
author's [inFAMOUS 2 port](https://github.com/erikomis/infamous2-recomp). See
[CREDITS.md](CREDITS.md).

> [!WARNING]
> **Early work in progress.** Chapter 1 can be played, with rendering bugs
> still open. The [releases](#releases--download) carry only the launcher:
> you build the game yourself from your own disc (the [launcher](#launcher)
> can run the rebuild and the game for you).
> Apple-silicon macOS only. A controller is the tested way to play; keyboard
> and mouse are new and still need a play test. What is missing is in
> [ROADMAP.md](ROADMAP.md).

> **This repository contains no game data, no firmware, no Sony keys and
> nothing translated from them.** Everyone generates all of that locally from
> their own legitimate copy. A fan project, not affiliated with Sony
> Interactive Entertainment or Insomniac Games.

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
[launcher/README.md](launcher/README.md#windows-preview).

The first release, **v0.1.0** (2026-10-23, a pre-release), is out: the macOS
launcher and the Windows preview. The settings added since (controls, window
modes, filters, texture packs) are in the sources and come with the next
release; build the launcher yourself to get them now
(`tools/rebuild.sh launcher`).

## Status

As of 2026-10-23 (the campaign at the game's own frame-rate cap; keyboard, mouse and the inFAMOUS 2 settings ported), every ✅ item verified in runs on an Apple-silicon Mac with Metal; 🧪 = built and tested, not yet verified in a play session.

| | |
| --- | --- |
| Both executables lifted to C++ and built (`EBOOT.BIN`: 25,022 functions; `game.self`: 18,994), and the v1.7 update's `game.self` (19,706 lifted functions, 19,219 for 01.00's) | ✅ |
| Boot, legal screens | ✅ (text from an earlier legal screen overlaps the next one) |
| First-run "initial setup" HDD install (writes the `*.done` cache files) | ✅ |
| Title screen, main menu (Campaign / Cooperative / Multiplayer / Options) | ✅ |
| Campaign difficulty and invert-Y prompts | ✅ |
| Saves and profile (`quick.sav`, `PROFILE.SAV`) written and read | ✅ |
| EBOOT → `game.self` hand-off (`sys_game_process_exitspawn`) into the level-1 intro movie | ✅ |
| Movies (cellVdec through FFmpeg, drawn as an immediate-mode quad) | ✅ the level-1 intro plays to the end |
| Raw SPUs (two programs `game.self` starts after the intro) | ✅ lifted and running |
| Chapter 1 ("The Gauntlet", York) loading screen | ✅ up to "Press the START button" |
| Gameplay | ✅ chapter 1 (York) played in a window with a DualSense: first person, HUD, enemies and allies, shooting, dying and respawning |
| Frame rate | ✅ the campaign reaches the game's own ~30–33 fps cap (Resistance runs at 30 fps on the PS3); the PPU's remaining "busy" time is largely the game's frame limiter. Since 2026-10-22: the game's 2560×720 depth buffer, which it samples as A8R8G8B8 every frame, is resolved on the GPU (it was a GPU→CPU readback and a 7.4 MB re-upload per frame): RSX thread 54% → 12% busy, texture re-uploads per 120 frames ~100 → 0; see [How it works](#how-it-works) |
| 60 fps and a wider field of view | ⏳ in progress (the community's RPCS3 patches, carried over to the port) |
| World colours (buildings, roofs, streets) | ✅ correct: the blue world was a vertex-offset wrap the RSX does and the port did not (see [How it works](#how-it-works)) |
| Shadows and smoke | ✅ the dark rectangles on the ground and the hard-edged smoke are gone in game (checked visually) after the depth fixes from a comparison with RPCS3: the shadow depth comparison ran in the wrong direction, depth textures now feed every lane (soft particles, smoke), depth snapshots are cropped to the region the texture names |
| Respawn | ✅ after dying the player comes back on the ground where the checkpoint is (it used to be sent up into the sky; a raw-SPU restart the runtime dropped, see [How it works](#how-it-works)); verified over 12 deaths |
| Window | ✅ one persistent window from the menus through the campaign: the small `Resistance` launcher owns it and the game draws into it through shared IOSurfaces, so the menu → campaign hand-off no longer closes it (~0.35 ms GPU per frame, no CPU copy); the title's guest FPS counts the game's flips |
| SPU jobs (geometry, draw sort, effects, NextSynth audio) | ✅ the 15 WWS job modules lifted |
| After the intro movie | ✅ the old intermittent hang is fixed (shared callback stack) |
| Controller | ✅ DualSense, DualShock or Xbox through SDL2 (input with the game process in the background is enabled for the launcher; not yet tested with a physical controller that way); 🧪 rumble (`PS3_RUMBLE`, `PS3_RUMBLE_STRENGTH`) ported, needs a play test |
| Keyboard and mouse | 🧪 ported from the inFAMOUS 2 work: an FPS layout (WASD, mouse look, left click fires), remappable in `input.toml` (`PS3_INPUT_MAP`, the launcher's Controls page); tests pass, needs a play test |
| Window modes, vsync, MetalFX, SMAA and filters, texture packs, internal resolution | 🧪 ported from the inFAMOUS 2 work (see [Environment](#environment)); tests pass, needs a play test; the internal resolution is experimental |
| Game update v1.7 | ✅ the campaign runs the update's `game.self`, re-lifted from `patch_06`: menus → New campaign → the York intro, gameplay and respawn verified; the menus stay the disc's 01.00 EBOOT (see [The v1.7 campaign](#the-v17-campaign)); rumble with a physical controller, a long session and a v1.7 fps measurement not verified yet |
| Launcher app | ✅ [`launcher/`](launcher/) (Tauri): disc, builds, rebuild, settings, saves, Play/Stop, in English and Brazilian Portuguese; 🧪 the new Controls page (remapping, mouse, rumble) and Graphics settings (window, filters, texture packs) |
| Linux and Windows | ⏳ not supported |
| Online multiplayer | ❌ out of scope (the official servers are gone) |

Details and next steps in [ROADMAP.md](ROADMAP.md).

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
  SPURS loads it ([tools/regen_spu_pm.sh](tools/regen_spu_pm.sh)).
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
  raw SPU 1 (program `5A10969E466A2298`), proxy-DMAs the program back into
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

The toolkit changes are one patch over ps3recomp v0.17.0,
[patches/ps3recomp-a6d76c8-resistance.patch](patches/ps3recomp-a6d76c8-resistance.patch):
the main-thread stack information, the SPURS kernel context the policy module
reads, the 28-bit vertex offset, the vertex cache and the rest of the
performance work, the fixes from a comparison with RPCS3 (shadow comparison
direction, depth snapshots cropped and released, depth replicated to every
lane, compressed textures never resolved from render targets, PPU threads
starting with VSCR[NJ]=1, no `getenv` on every SPU MFC channel write), the
raw-SPU stop and restart through `SPU_RunCntl`, the shared-IOSurface window
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

### What you need

- An Apple-silicon Mac (macOS with Metal).
- Your own **Resistance: Fall of Man disc, `BCUS98107` v01.00**, dumped to an
  image and mounted (the folder that holds `PS3_GAME/`).
- [RPCS3](https://rpcs3.net) to decrypt the two executables (if decryption
  fails, install the PS3 firmware in RPCS3 first). ELFs you decrypted some
  other way work too: drop them in as shown below.
- Xcode Command Line Tools and Homebrew:

```sh
brew install cmake ninja sdl2 glslang spirv-cross spirv-tools ffmpeg python@3.12
```

### Building

```sh
git clone https://github.com/erikomis/resistance-recomp.git
cd resistance-recomp
tools/setup_toolkit.sh                        # ps3recomp v0.17.0 + patch in ./ps3recomp
python3.12 -m venv .venv
.venv/bin/pip install -r ps3recomp/requirements.txt
PY=.venv/bin/python PS3_VFS_ROOT=/Volumes/<your disc> tools/rebuild.sh
```

`tools/rebuild.sh` runs these steps (or only the ones you name, e.g.
`tools/rebuild.sh lift build`):

| Step | What it does | Writes |
| --- | --- | --- |
| `toolkit` | `tools/setup_toolkit.sh` | `ps3recomp/` |
| `decrypt` | copies `EBOOT.BIN` and `game.self` from the disc and runs `rpcs3 --decrypt` (`RPCS3=<path>` if it is not in `/Applications`); skipped when the `.elf` is already there; checks the SHA-256 of the result against the tested v01.00 executables | `game/BCUS98107/EBOOT.elf`, `game/BCUS98107/modules/game.elf` |
| `analyse` | the toolkit's `ppu_loader.py` (functions, imports, TOC) | `analysis/BCUS98107/` |
| `lift` | the toolkit's `ppu_lifter.py`, 16 chunks for the EBOOT and 12 for `game.self` | `recompiled/BCUS98107/`, `recompiled/BCUS98107/game_module/` |
| `spu` | `tools/regen_spu_pm.sh`: the job-manager policy module, extracted from your EBOOT, wrapped and lifted | `port/spu/captured/` |
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

After the level-1 intro, `game.self` streams two programs into raw SPUs by
proxy DMA (local store 0x60 onward, entry 0x90). They are captured from a run,
then lifted at base 0x60:

```sh
SPU_RAW_DUMP_DIR=$PWD/out/rawspu tools/to_campaign.sh rawspu   # then wait ~5 min for "[spu-raw] dumped -> ..." twice
tools/regen_rawspu.sh out/rawspu                          # -> port/spu/rawspu0, port/spu/rawspu1
```

`port/spu/spu_workloads.c` registers them as SPU images 2
(`1894F66385AFD052`) and 3 (`5A10969E466A2298`). Like the policy module, the
lifted C is translated game code and stays out of git.
`port/spu/spu_workloads.c` registers the lifted entry
(`resistance_spu_spu_func_00000A00`) under that fingerprint as SPU image 1.

#### The v1.7 campaign

The game's last update, v1.7, is seven archives, `patch_01`…`patch_07.psarc`;
`patch_06` carries a new campaign executable (`/data/new_game.self`). The
port runs it. Its decrypted ELF (SHA-256 `e1d75e6d4afe…144f`, RPCS3's
"game.self Updated", PPU-a08a187d…) is lifted like the disc's (19,706
functions against 19,219), and 13 of the 15 WWS job modules and both raw-SPU
programs changed and are lifted from it (new fingerprints `6D03497950639952`
and `47C9CB480016D493`; the policy module is byte-identical). It imports
`cellSaveDataAutoSave2`/`AutoLoad2`, `_sys_process_atexitspawn` and
`cellPadSetActDirect` (rumble), which 01.00 does not.

The patch files are the game's official free updates and are not in this
repository. If you own the game, [PSRewired](https://psrewired.com) lets it
download its own update in RPCS3; copy `patch_01`…`patch_07.psarc` from
RPCS3's `dev_hdd0/game/BCUS98107/USRDIR/data` to `update/` in this checkout
(or point `RES_UPDATE_DIR` at them), then:

```sh
tools/rebuild.sh v17                                  # tools/v17/regen_v17.sh
SPU_RAW_DUMP_DIR=$PWD/out/rawspu_v17 tools/play.sh    # once: new campaign, let the intro movie end, close the window
tools/v17/regen_v17.sh spu build                      # lifts the two raw-SPU programs just captured
```

`regen_v17.sh` extracts `new_game.self` from `patch_06`
(`tools/v17/psarc_extract.py`), decrypts it with `rpcs3 --decrypt` into
`game/BCUS98107/modules_v17/game.elf` (checking the SHA-256), lifts it to
`recompiled/BCUS98107/game_module_v17/`, lifts its SPU code into
`port/spu_v17/` (the policy module is copied from `port/spu/captured/`, so
run the `spu` step first) and builds `build/BCUS98107_game_v17/`. The raw-SPU
programs are captured from one run, as for 01.00; until they are lifted the
v1.7 campaign stops after the intro movie.

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

## Launcher

[`launcher/`](launcher/) is a desktop app (Tauri) that does what
`tools/play.sh` does without a terminal, modelled on the inFAMOUS 2 port's
launcher: it finds your builds and your disc, mounts the disc image for you
(`hdiutil`, read-only) and remembers it, keeps the settings, and **Play**
starts the `Resistance` window host with the EBOOT build, the play folder
(`PS3_HDD0_ROOT`/`PS3_HDD1_ROOT`, `last.log`), FFmpeg and `caffeinate`, as
`play.sh` does. **Stop** ends the game and its window (the window host now
exits when the game does). It ships nothing from the game: the cover on its home screen
is read from your mounted disc.

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
  60 fps and field-of-view options, shown once the build has them (in
  progress).
- **Game**: the disc (image or folder, mount/eject), the campaign version
  (**v1.7** when its build is there, or **01.00**: `RES_GAME_VERSION=100`),
  the console language (`PS3_LANG`), movies (FFmpeg paths), the play folder,
  and a saves manager (the profile and `quick.sav`, here and in RPCS3:
  backups before every Play, restore, copy either way, .zip export/import).
- **Controls**: the bindings for keyboard, mouse and controller with what
  each button does in the game (written to `input.toml`), a controller test,
  what drives the game (both, controller only, keyboard and mouse only), mouse
  sensitivity, inversion, dead zone and capture, vibration and its strength.
  Every setting reaches the game only when it differs from the default.
- **Advanced**: Install / rebuild (requirements with Homebrew installs, the
  port folder, `tools/rebuild.sh` with chosen steps and a progress bar),
  Updates (`git fetch` of the checkout, "builds older than the sources", the
  newest launcher release), the game log viewer, debug mode (a session folder
  with captures, a log summary and a .zip to attach to an issue), and the raw
  environment with the equivalent command line.

UI in Brazilian Portuguese and English. Build it with Node 20+ and Rust:

```sh
tools/rebuild.sh launcher        # -> build/launcher/Resistance Launcher.app
```

Details, screenshots and the development commands are in
[launcher/README.md](launcher/README.md).

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

The scripts set these for you ([tools/env.sh](tools/env.sh)); to run a build
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
| `PS3_INPUT_MAP=<file>` | keyboard, mouse-button and controller bindings and the `[mouse]` settings (`input.toml`; format in the toolkit's `libs/input/pad_input_map.h`) |
| `PS3_INPUT_MODE=both\|controller\|keyboard` | what drives the game (default both); `PS3_PAD_SDL=0` ignores physical controllers |
| `PS3_MOUSE=0`, `PS3_MOUSE_SENS=<f>`, `PS3_MOUSE_INVERT_X=1`, `PS3_MOUSE_INVERT_Y=1`, `PS3_MOUSE_DEADZONE=0..120`, `PS3_MOUSE_CAPTURE=0` | the mouse camera: off, speed (1.0), inversion, first stick step past centre (30), no cursor capture |
| `PS3_RUMBLE=0`, `PS3_RUMBLE_STRENGTH=0..200` | vibration off / its strength in percent (100) |
| `PS3_WINDOW_MODE=windowed\|fullscreen\|borderless`, `PS3_WINDOW_SIZE=WxH`, `PS3_DISPLAY=<n\|name>`, `PS3_VSYNC=0`, `PS3_UPSCALE=metalfx` | the window host: how the window opens (letterboxed; `--size` still works), on which screen, vsync off, MetalFX upscaling |
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
| [`port/`](port/) | The port: `main.cpp` (boot, guest thread, exitspawn data block), `stubs.cpp` (exitspawn and the window-state hand-off, `RES_PEEK`), `CMakeLists.txt` (with the `Resistance` launcher target), `config.toml`, `spu/spu_workloads.c` (SPU registration); `spu_v17/` is generated by `tools/v17/regen_v17.sh` |
| [`patches/`](patches/) | The toolkit patch over ps3recomp v0.17.0 |
| [`tools/`](tools/) | `setup_toolkit.sh`, `rebuild.sh`, `regen_spu_pm.sh`, `extract_pm.py`, `play.sh` and the run scripts; `v17/` (the v1.7 campaign: `regen_v17.sh`, `psarc_extract.py`, `spu_workloads.c`) |
| [`launcher/`](launcher/) | The desktop launcher (Tauri): settings, disc, saves, rebuild and Play |
| [`ROADMAP.md`](ROADMAP.md) | Status and next steps |

## License

Code in this repository: MIT ([LICENSE](LICENSE)). The toolkit is MIT by its
authors. Resistance: Fall of Man © Sony Interactive Entertainment LLC,
developed by Insomniac Games; this project is not affiliated with Sony or
Insomniac.
