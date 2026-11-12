// A fake Tauri backend so the built frontend can be looked at in a plain
// browser (the screenshots in docs/screenshots). Injected before the page
// loads:
//
//   npm run build && npx vite preview --port 4173
//   # then, e.g. with Playwright: page.addInitScript({ path: "scripts/ui-mock.js" })
//
// It answers the commands the window calls with a built port, a mounted disc,
// saves and a log, and no art (the art only ever comes from the player's
// disc). window.__UI_MOCK__ = { noBuilds: true }, { noDisc: true },
// { running: true } or { updateFiles: true }, set by an earlier init script,
// shows those states instead; { oldBuild: true } is a build from before the
// keyboard/mouse, filters and window modes, { patches: true } one that has the
// 60 fps / FOV patches, { appUpdate: true } a newer launcher on offer and
// { updatedFrom: true } the start after an update (rebuild offered).
(() => {
  const flags = window.__UI_MOCK__ || {};
  const home = "/Users/player";
  const data = `${home}/Library/Application Support/resistance-recomp`;
  const settings = {
    uiLang: flags.lang || "pt-BR", portDir: "", discPath: "", discImage: `${home}/Games/Resistance.iso`, autoMount: true, playDir: "",
    campaign: "auto", gameLang: "", windowSize: "1920x1080", gameIcon: false, aa: "fxaa", sharpen: 0.2, aniso: 4,
    gammaOverride: false, gamma: 1, showFps: true, vertexCache: true, vertexCacheSplit: true, bindChangedOnly: true,
    shaderCache: true, shaderAsync: false, frameClockUs: 1000, keepAwake: true, video: true, ffmpegAvcodec: "", ffmpegAvutil: "",
    quietLog: true, extraEnv: "", rebuildSteps: [], jobs: 0, saveBackupsKeep: 10, autoUpdateCheck: true,
    windowMode: "windowed", display: "", vsync: true, upscale: "", resScale: "", vibrance: 0, contrast: 1, vignette: 0, grain: 0,
    lutPath: "", lutStrength: 1, mods: ["HD Chimera"], modsDump: false, inputMode: "both", mouse: true, mouseSens: 1,
    mouseInvertY: false, mouseInvertX: false, mouseDeadzone: 30, mouseCapture: true, rumble: true, rumbleStrength: 100,
    audioVolume: 100, audioMute: false, audioDevice: "",
    unlockFps: false, menu60fps: false, fov: "",
  };
  const features = flags.oldBuild ? [] : ["PS3_INPUT_MAP", "PS3_LUT_STRENGTH", "PS3_MODS_DIR", "PS3_RES_SCALE", "PS3_UPSCALE", "PS3_WINDOW_MODE"]
    .concat(flags.patches ? ["RES_FOV", "RES_MENU_60FPS", "RES_UNLOCK_FPS"] : []);
  const now = Math.floor(Date.now() / 1000);
  const root = `${home}/Developer/resistance-recomp`;
  const builds = {
    root, host: `${root}/build/BCUS98107/Resistance`, eboot: `${root}/build/BCUS98107/ResistanceRecomp`,
    game: `${root}/build/BCUS98107_game/ResistanceRecomp`, gameV17: `${root}/build/BCUS98107_game_v17/ResistanceRecomp`,
    ok: true, v17: true, legacy: false, missing: [], rebuildSh: `${root}/tools/rebuild.sh`, builtAt: now - 3600 * 3,
  };
  const disc = flags.noDisc
    ? { root: null, title: "", titleId: "", appVer: "", image: null, ours: false, mounted: [] }
    : { root: "/Volumes/RESISTANCE_1", title: "Resistance: Fall of Man", titleId: "BCUS98107", appVer: "01.00", image: `${home}/Games/Resistance.iso`, ours: true, mounted: ["/Volumes/RESISTANCE_1"] };
  const slot = (name, kind, sub, mod, hash) => ({
    name, kind, title: kind === "profile" ? "Resistance: Fall of Man" : "", subtitle: sub, detail: "", modified: mod,
    bytes: kind === "profile" ? 90000 : 52800, path: `/saves/${name}`, icon: null, hash,
  });
  const log = [
    "[host window] 'Resistance: Fall of Man' pid 4242 window 812, game pid 4243",
    "[cellSysutil] system language 1 (host)",
    "[RSX metal] host window: 5 surfaces 1280x720 (generation 1)",
    "[cellVdec] FFmpeg libavcodec loaded",
    "[fs] open FAIL /dev_hdd0/game/BCUS98107/USRDIR/data/patch_01.psarc",
    "[RSX FPS] display=30.00 guest=30.00 avg_ms=33.30 p95_ms=34.10 max_ms=36.00 age_s=0.00",
    "[hle] stub cellNetCtlGetInfo (offline)",
    "[exitspawn] campaign = v1.7 game.self",
    "[RSX FPS] display=31.00 guest=31.00 avg_ms=32.20 p95_ms=33.40 max_ms=35.10 age_s=0.00",
  ].join("\n") + "\n";
  const answers = {
    get_settings: () => settings,
    save_settings: () => null,
    get_status: () => ({
      builds: flags.noBuilds ? { ...builds, ok: false, host: null, eboot: null, game: null, gameV17: null, v17: false, missing: ["build/BCUS98107/Resistance", "build/BCUS98107/ResistanceRecomp", "build/BCUS98107_game/ResistanceRecomp"], builtAt: 0 } : builds,
      disc, icon: null, background: null,
      lastSave: { name: "quick.sav", title: "", subtitle: "", modified: now - 3600 * 5, folder: "/saves" },
      gameRunning: !!flags.running, otherGameRunning: false, rebuildRunning: false, playDir: data, dataRoot: data,
      firstBoot: false, updateFiles: flags.updateFiles ? [`${data}/dev_hdd0/game/BCUS98107/USRDIR/data/patch_06.bin`] : [],
      hostSupported: true, experimental: false, os: "macos", appBundle: "/Applications/Resistance Launcher.app", finderIcon: false, version: "0.1.0",
      features,
    }),
    disc_status: () => disc,
    find_port_dirs: () => [builds],
    check_updates: (a) => ({
      sources: { git: true, branch: "main", commit: "1a2b3c4", subject: "Launcher", commitTime: now - 7200, upstream: "origin/main", behind: 0, ahead: 0, dirty: 0, sourcesTime: now - 7200, fetched: !!a.online, error: null },
      rebuildNeeded: false, builtAt: builds.builtAt,
      app: flags.appUpdate
        ? { current: "0.2.2", checked: !!a.online, latest: "0.3.0", newer: !!a.online, url: "https://github.com/erikomis/resistance-recomp/releases/tag/v0.3.0", notes: "Launcher v0.3.0.", pubDate: null, disabled: false, error: null, installBlocker: null, installedPending: null, updatedFrom: null }
        : { current: "0.2.2", checked: !!a.online, latest: "0.2.2", newer: false, url: null, notes: null, pubDate: null, disabled: false, error: null, installBlocker: null, installedPending: null, updatedFrom: flags.updatedFrom ? "0.2.1" : null },
    }),
    install_update: () => "0.3.0",
    restart_app: () => null,
    steam_status: () => ({ available: true, installed: true, running: false, users: 1, added: 0 }),
    shader_cache_info: () => ({ path: `${data}/shader-cache`, bytes: 18400000, files: 1240 }),
    report_info: () => (flags.running ? { dir: "/r/x", id: "20261022-190000", live: true, debug: true, captures: 1, pending: false, passes: 64, bytes: 210000000, logBytes: 9000000, duration: 300, exit: null, stopped: false, crashReport: false, zip: null, zipBytes: 0, triage: null } : null),
    report_list: () => [
      { dir: "/r/20261022-181500", id: "20261022-181500", live: false, debug: true, captures: 2, pending: false, passes: 64, bytes: 286000000, logBytes: 41200000, duration: 1874, exit: "signal=15", stopped: true, crashReport: false, zip: null, zipBytes: 0,
        triage: { errors: 3, warnings: 41, lines: 120000, fpsAvg: 30.4, fpsMin: 21.5, stale: false, md: "/r/triage.md",
          top: [
            { severity: "error", title: "[fs] open FAIL /dev_hdd#/game/BCUS#/USRDIR/data/patch_#.psarc", count: 3, firstLine: 812 },
            { severity: "warning", title: "[hle] stub cellNetCtlGetInfo (offline)", count: 38, firstLine: 1204 },
          ] } },
    ],
    check_prereqs: () => ({
      supported: true, clt: true,
      checks: ["xcode-clt", "deps", "rpcs3", "disc", "toolkit", "venv", "disk", "builds"].map((n) => ({ level: "ok", name: n, message: n === "disk" ? "42 GB free, about 0 GB needed" : n === "rpcs3" ? "not needed (optional)" : "ok" })),
      ok: true, deps: true, depsDir: `${data}/deps`, sources: root, elfs: !flags.noBuilds, disc: !flags.noDisc,
      buildsOk: !flags.noBuilds, freeGb: 42, needGb: flags.noBuilds ? 4 : 0,
    }),
    saves_overview: () => ({
      port: [slot("BCUS98107_PROFILE", "profile", "Profile", now - 3600 * 5, "a1"), slot("quick.sav", "checkpoint", "", now - 3600 * 5, "c1")],
      rpcs3: [slot("BCUS98107_PROFILE", "profile", "Profile", now - 86400 * 9, "a0"), slot("quick.sav", "checkpoint", "", now - 86400 * 9, "c0")],
      rpcs3Users: ["00000001"], rpcs3User: "00000001",
      backups: [{ id: "20261022-171500-port-auto", created: now - 3600 * 6, reason: "auto", side: "port", user: "", slots: ["BCUS98107_PROFILE", "quick.sav"], titles: { BCUS98107_PROFILE: "Profile" }, bytes: 142800 }],
      portDir: "/x/savedata", portUsrdir: "/x/USRDIR", rpcs3Dir: "/x/rpcs3", backupsDir: "/x/save-backups", gameRunning: false, rpcs3Running: false,
    }),
    launch_preview: () => `cd '${data}' && PS3_VFS_ROOT=/Volumes/RESISTANCE_1 PS3_HDD0_ROOT='${data}/dev_hdd0' PS3_HDD1_ROOT='${data}/dev_hdd1' PS3RECOMP_FFMPEG_AVCODEC=/opt/homebrew/lib/libavcodec.dylib PS3RECOMP_FFMPEG_AVUTIL=/opt/homebrew/lib/libavutil.dylib PS3_AA=fxaa PS3_SHARPEN=0.20 PS3_ANISO=4 PS3RECOMP_METAL_FPS=1 PS3_SHADER_CACHE='${data}/shader-cache' PS3_VERBOSE=0 /usr/bin/caffeinate -i ${root}/build/BCUS98107/Resistance --size 1920x1080 -- ${root}/build/BCUS98107/ResistanceRecomp ${root}/game/BCUS98107/EBOOT.elf > '${data}/last.log' 2>&1`,
    read_game_log: () => ({ path: `${data}/last.log`, exists: true, size: log.length, modified: now - 120, text: log, truncated: false }),
    log_paths: () => ({ game: `${data}/last.log`, saves: "/x/savedata", play: data, data }),
    take_play_request: () => false,
    take_online_request: () => false,
    take_start_view: () => null,
    get_input_map: () => ({ path: `${data}/input.toml`, exists: false, bindings: { keyboard: {}, mouseButtons: {}, controller: {} } }),
    save_input_map: () => `${data}/input.toml`,
    reset_input_map: () => null,
    list_screens: () => ["Built-in Retina Display", "LG ULTRAFINE"],
    list_mods: () => ({
      dir: `${data}/mods`, dumpDir: `${data}/texture-dump`,
      packs: [{ name: "HD Chimera", textures: 214, description: "Sharper Chimeran armour" }, { name: "Clean HUD", textures: 12, description: "" }],
    }),
  };
  let cb = 0;
  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd, args) => {
      if (cmd.startsWith("plugin:event|")) return cmd.endsWith("listen") ? ++cb : null;
      if (cmd.startsWith("plugin:dialog|")) return null;
      if (cmd === "save_settings") Object.assign(settings, args.settings);
      const f = answers[cmd];
      if (!f) {
        console.warn("ui-mock: no answer for", cmd, args);
        return null;
      }
      return f(args || {});
    },
    transformCallback: () => ++cb,
    unregisterCallback: () => {},
    convertFileSrc: (p) => p,
    metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
})();
