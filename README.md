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
shooting, dying and respawning. It runs at 7–8 fps, buildings render bright
blue and the player can end up above the map (see [Status](#status)).

Built on [ps3recomp](https://github.com/sp00nznet/ps3recomp), following the
author's [inFAMOUS 2 port](https://github.com/erikomis/infamous2-recomp). See
[CREDITS.md](CREDITS.md).

> [!WARNING]
> **Early work in progress.** Chapter 1 can be played, but slowly and with
> rendering bugs. There is no release and no app: you build it yourself from
> your own disc. Apple-silicon macOS only, and a controller is required. What
> is missing is in [ROADMAP.md](ROADMAP.md).

> **This repository contains no game data, no firmware, no Sony keys and
> nothing translated from them.** Everyone generates all of that locally from
> their own legitimate copy. A fan project, not affiliated with Sony
> Interactive Entertainment or Insomniac Games.

## Status

As of 2026-10-22 (chapter 1 played with a controller), every item verified in runs on an Apple-silicon Mac with Metal.

| | |
| --- | --- |
| Both executables lifted to C++ and built (`EBOOT.BIN`: 25,022 functions; `game.self`: 18,994) | ✅ |
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
| Frame rate | 🚧 7–8 fps windowed (frames of ~130–146 ms): the RSX thread is ~100% busy decoding vertices on the CPU (`dc_fetch` ~41%, `rsx_vertex_remap_build` ~13%) and the rest of the game waits on it; a cross-frame vertex cache (from the inFAMOUS 2 port) is in progress |
| Buildings and roofs | 🚧 drawn solid bright blue (under investigation with per-pass dumps) |
| Shadows | 🚧 dark/black rectangles on the ground (not investigated yet) |
| Player position | 🚧 after dying and respawning the view is from above the town (seen from the sky, inside-out); in a headless run walking forward also ended on the rooftops (not investigated yet) |
| SPU jobs (geometry, draw sort, effects, NextSynth audio) | ✅ the 15 WWS job modules lifted |
| After the intro movie | ✅ the old intermittent hang is fixed (shared callback stack) |
| Controller | ✅ DualSense, DualShock or Xbox through SDL2 |
| Keyboard and mouse | ⏳ not on macOS (the toolkit's keyboard fallback is Windows-only): a controller is required |
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

The toolkit changes are one patch over ps3recomp v0.17.0,
[patches/ps3recomp-a6d76c8-resistance.patch](patches/ps3recomp-a6d76c8-resistance.patch):
the main-thread stack information, the SPURS kernel context the policy module
reads, and a few diagnostics (`VDEC_PICLOG`, `PPU_STACKINFO_LOG`,
`SPURS_PM_R4_WORD0`, `SPURS_PM_TRACE`, `RSX_ENGINE_PASS_DUMP_TRIGGER`,
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
| `build` | CMake + Ninja, Metal, FFmpeg runtime | `build/BCUS98107/MyGameRecomp`, `build/BCUS98107_game/MyGameRecomp` |

Everything in that last column is generated from your copy and is
`.gitignore`d. Keep the layout: when the EBOOT hands off to the campaign,
`port/stubs.cpp` runs `build/BCUS98107_game/MyGameRecomp` with
`game/BCUS98107/modules/game.elf` next to the EBOOT build
(`RES_SPAWN_BIN` / `RES_SPAWN_ELF` override that).

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

## Playing

Mount your disc image, plug in or pair a controller (DualSense, DualShock or
Xbox; on macOS keyboard and mouse do not work yet), then:

```sh
tools/play.sh
```

It starts the front end in a window; starting a campaign hands off to the
`game.self` build in the same window. On the **first boot press START
twice**: on the legal screen, then on the "initial setup" prompt (the HDD
install, done once). Saves, the install cache and the log (`last.log`) are
kept in `~/Library/Application Support/resistance-recomp/` (`RES_DATA`
changes that). The window title shows the frame rate. Close the window, or
press Ctrl+C in the terminal, to quit. If a build is missing, `play.sh` says
so: run `tools/rebuild.sh`.

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
directly: `build/BCUS98107/MyGameRecomp game/BCUS98107/EBOOT.elf`.

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
| `SPURS_PM_TRACE=1` | log the job manager's policy-module polls (`[spurs-pm]`; off by default, it costs frame rate) |
| `PS3_TITLE` | window title |
| `RES_SPAWN_BIN`, `RES_SPAWN_ELF` | the build and ELF the exitspawn hand-off runs (default: the sibling build directory and `game/BCUS98107/…`) |
| `RES_SPAWN_KEEP=1` | keep the exitspawn data file (to rerun `game.self` by hand with `RES_SPAWN_DATA=<file>`) |
| `RES_PEEK=<secs>:<addr>[,*<addr>...]` | every `<secs>`, print 32 bytes of guest memory at each address (`*` dereferences first) |
| `SPU_DUMP_MISS=<dir>` | write every SPU image/policy module that has no lifted code to `<dir>` |
| `VDEC_PICLOG=1`, `PPU_STACKINFO_LOG=1` | log the first decoded pictures / every stack-information query |

## Layout

| Path | What it is |
| --- | --- |
| [`port/`](port/) | The port: `main.cpp` (boot, guest thread, exitspawn data block), `stubs.cpp` (exitspawn, `RES_PEEK`), `CMakeLists.txt`, `config.toml`, `spu/spu_workloads.c` (SPU registration) |
| [`patches/`](patches/) | The toolkit patch over ps3recomp v0.17.0 |
| [`tools/`](tools/) | `setup_toolkit.sh`, `rebuild.sh`, `regen_spu_pm.sh`, `extract_pm.py`, `play.sh` and the run scripts |
| [`ROADMAP.md`](ROADMAP.md) | Status and next steps |

## License

Code in this repository: MIT ([LICENSE](LICENSE)). The toolkit is MIT by its
authors. Resistance: Fall of Man © Sony Interactive Entertainment LLC,
developed by Insomniac Games; this project is not affiliated with Sony or
Insomniac.
