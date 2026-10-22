# Roadmap

🌐 **English** · [Português (BR)](ROADMAP.pt-BR.md)

Status of the native Resistance: Fall of Man (`BCUS98107`, disc, v01.00) port
to macOS/arm64 and what comes next. Same layout as the inFAMOUS 2 roadmap.
`[x]` = done and verified; `[ ]` = to do; "(in progress)" = being worked on.

## Status at a glance (2026-10-21)

- **Runs:** both executables (`EBOOT.BIN` and `game.self`) are lifted, build and run on macOS/arm64 with Metal. The front end goes from boot through the legal screens, the first-run HDD install, the title screen and the main menu (Campaign / Cooperative / Multiplayer / Options) to the campaign difficulty and invert-Y prompts; saves (`PROFILE.SAV`, `quick.sav`) are written and read back. Starting a campaign exitspawns `game.self`, which boots straight into the level-1 intro movie.
- **Blocked at:** the in-game movies show black (the picture is drawn as an immediate-mode quad; under investigation), so the intro cannot be watched yet and gameplay has not been reached.
- **Released:** nothing yet.
- **In progress:**
  - [ ] (in progress) Black movies: cellVdec decodes the pictures (FFmpeg runtime) but the immediate-mode quad that should show them puts nothing on screen.
  - [ ] (in progress) Text from earlier legal screens stays on screen and overlaps the next ones.
- **Next:** the intro movie on screen, then the first chapter (York) in a window.

## Current state (2026-10-21)

The whole pipeline runs from the player's own disc (`tools/rebuild.sh`): the
disc image is mounted and used as the VFS root (`PS3_VFS_ROOT`), both PPU
executables are decrypted (`game/BCUS98107/EBOOT.elf`,
`game/BCUS98107/modules/game.elf`), analysed, lifted (EBOOT 25,022
functions; `game.self` 18,994) and built against ps3recomp v0.17.0 plus this
repository's toolkit patch.

What got the front end on screen:

- **Job manager:** the game's SPURS job-manager policy module (fingerprint
  `4E4076E700C6AFF3`, 11,136 bytes) is lifted at its real local-store base,
  0xA00 (`tools/regen_spu_pm.sh`). Before that it spun for ever on
  `GETLLAR 0x2480` and the main thread waited on it in `usleep(30)`.
- **Metal off the guest thread:** the guest runs on its own thread and the
  macOS main thread turns the run loop (`port/main.cpp`). With the guest on the
  main thread the Metal backend's event pump deadlocked and the FIFO never
  drained.
- **Main-thread stack:** `sys_ppu_thread_get_stack_information` reported
  0 + 0 for the main thread; it now reports the real stack (toolkit patch).
- **exitspawn:** `sys_game_process_exitspawn` is implemented in
  `port/stubs.cpp` as an `execv` of the other build, with the parent's data
  block placed at main-stack-top − 0x1000, where `game.self` looks for its
  `"ELFtastic!"` block.
- **Video:** cellVdec decodes through FFmpeg loaded at runtime
  (`-DPS3RECOMP_FFMPEG_RUNTIME=ON`, Homebrew FFmpeg).

The SPU jobs the game loads at runtime (the NextSynth audio jobs and others)
still run in the toolkit's SPU interpreter.

---

## What is missing to reach gameplay

### Blockers

- [ ] **Movies are black.** The level-1 intro (and the front-end movies) decode, but the quad that presents the picture is drawn in immediate mode and nothing reaches the frame. Find what the draw needs (vertex data through the immediate-mode path, the texture the picture is uploaded to, its format) and compare with RPCS3.
- [ ] **Legal-screen text overlaps.** Text drawn on one legal screen is still there on the next: find what clears (or should clear) the previous text.
- [ ] `[vm] OOB access 0xFFFFF000 (+1)` right after the `parameters.txt` probes: check whether it went away with the stack-information fix, and if not, find the guest PC (`PS3_NULL_READ_STACK=1`).

### Into the first chapter

- [ ] Get through (or skip) the intro movie into York.
- [ ] Loading screens and the first level streaming from `game.psarc` (6.26 GB).
- [ ] Pause menu and in-game options.

### SPU jobs

- [ ] SPU jobs the game loads at runtime: capture each new fingerprint (`SPU_DUMP_MISS=<dir>`, as `spujob_4E4076E700C6AFF3_11136` was), lift it, register it. Start with the NextSynth audio jobs (`ps3soundstream`).
- [ ] A script that lifts every captured job in one go (inFAMOUS 2's `lift_spu_jobs.sh`, `relift_jobmod.py`, `lift_relocmods.py` are the model).

### Video and audio

- [ ] Movie timing and A/V sync once the picture shows.
- [ ] Audio: `cellAudio` ports and the notify event queue; the sound-stream SPU jobs above. Record with `AUDIO_WAV` and listen.

### Menu

- [x] Title screen and main menu.
- [ ] Options, `cellOskDialog` (on-screen keyboard), `cellMsgDialog`.
- [x] Saves: `PROFILE.SAV` and `quick.sav` written and read under `PS3_HDD0_ROOT`.
- [ ] New game into the first cutscene, watched to the end.

---

## What is missing to be 100% playable

### Gameplay

- [ ] The first chapter (York) with keyboard and with a controller, in a window.
- [ ] Play the whole campaign and write down what breaks: every chapter, the vehicles, the bosses, every weapon's alt-fire.
- [ ] Offline split-screen (co-op and versus): two pads, two viewports.

### Graphics

- [ ] Compare against the original (RPCS3 or the console) chapter by chapter; open an item per bug with a capture, as in inFAMOUS 2.

### Performance

- [ ] 30 fps (the game's own target) in gameplay. Profile per thread (`PS3_FTRACE`) as soon as there is gameplay.
- [ ] Measure how much SPU time is interpreted and cut it (translate what is left).

### Stability

- [ ] Long windowed sessions without hangs; a `watch_stall.sh`-style hang report.

### Controls

- [ ] Keyboard and mouse (the toolkit's `cellPad`), configurable mapping (`PS3_INPUT_MAP`).
- [ ] DualSense/DualShock/Xbox via SDL2, rumble.
- [ ] SIXAXIS: the game calls `cellPadSetSensorMode`/`cellPadInfoSensorMode`; find what it uses motion for and map it to a button.

### Rebuild

- [x] One script for the whole rebuild from the player's own disc (`tools/rebuild.sh`: decrypt with RPCS3, analyse, lift PPU and SPU, compile). The analysis and the lift it produces are byte-identical to the tested ones.
- [ ] Decryption without RPCS3 (inFAMOUS 2's `decrypt_self.py`, with the player's own key file).

---

## After it is playable (improvements)

### Graphics and resolution

- [ ] Internal resolution above 720p, MetalFX/FSR, widescreen FOV, fullscreen/borderless (reuse the inFAMOUS 2 toolkit patch: `PS3_WINDOW_MODE`, `PS3_DISPLAY`, `PS3_ANISO`, FXAA, CAS, `PS3_UPSCALE=metalfx`).
- [ ] Unlock past 30 fps where the game logic allows.

### App to install and play (launcher)

- [ ] The inFAMOUS 2 launcher (Tauri) adapted: install/rebuild from the disc image, settings, controls, saves, report a problem, updates.
- [ ] Icon from the player's own `ICON0.PNG`, Game Mode app bundle on macOS.

### Other platforms

- [ ] Linux (Vulkan) and Windows (D3D12), with the launcher and the rebuild running natively.

### Out of scope for now

- [ ] Online (40-player matches, `sceNp`, `sys_net`, `cellMic` voice chat): the official servers are gone; only with a community server.

---

## Done and finished

### Boot and pipeline

- [x] Disc dump `BCUS98107` v01.00 (PARAM.SFO `VERSION` 01.00), mounted as the VFS root.
- [x] `EBOOT.BIN` and `game.self` decrypted to ELF.
- [x] Analysis: `EBOOT` 25,022 functions, 183 imports in 14 libraries; `game.self` 18,994 functions, 134 imports in 11 libraries.
- [x] PPU lifted to C++ for both, and both projects build on macOS/arm64 with Metal.
- [x] Job-manager policy module (`4E4076E700C6AFF3`, 11,136 bytes; guest 0x10191A80 in the EBOOT, 0x10176E80 in `game.self`) lifted at LS base 0xA00 and registered as SPU image 1 (`port/spu/spu_workloads.c`).

### Front end

- [x] CRT and TLS, `sys_memory`, `cellSysmodule` loads, `cellDiscGameGetBootDiscInfo`, `cellSysCacheMount`, `cellGameDataCheckCreate2`.
- [x] `cellSpursInitialize` (3 SPUs), the job-manager workloads running, `cellGcm` init and flips on screen.
- [x] Guest off the macOS main thread (no more Metal event-pump deadlock).
- [x] Legal screens, the first-run "initial setup" HDD install (the `*.done` cache files), title screen, main menu.
- [x] Campaign difficulty and invert-Y prompts; `PROFILE.SAV` and `quick.sav` written and read.
- [x] `sys_game_process_exitspawn` EBOOT → `game.self` (an execv of the other build; the data block at main-stack-top − 0x1000), and `game.self` booting into the level-1 intro movie.
- [x] `sys_ppu_thread_get_stack_information` for the main thread.
- [x] cellVdec through the FFmpeg runtime.

---

## Giving back

- [ ] Toolkit fixes from this port as PRs to [ps3recomp](https://github.com/sp00nznet/ps3recomp), each with a test where possible (the main-thread stack information first).
- [x] Its own repository (`resistance-recomp`), as `infamous2-recomp`: port sources, a toolkit patch, `tools/setup_toolkit.sh`, README, credits, and a `.gitignore` that blocks game data, firmware and saves.
