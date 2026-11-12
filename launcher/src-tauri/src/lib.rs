//! Launcher core for the Resistance: Fall of Man (BCUS98107) port: the
//! settings, the player's disc (a mounted image or a folder), the builds of a
//! port checkout, the rebuild (tools/rebuild.sh) as a child process whose
//! output is streamed to the window, and "Play", which starts the window host
//! (`build/BCUS98107/Resistance`) running the EBOOT build, with the settings
//! mapped onto the port's environment variables, as tools/play.sh does. No
//! game code or data is part of the app: everything it shows or starts comes
//! from the player's own disc and build folder.
//!
//! Windows (preview, untested with the port itself): the disc is attached
//! with PowerShell's Mount-DiskImage and found over the drive letters, the
//! play folder is %APPDATA%\resistance-recomp, and Play starts
//! build\BCUS98107\ResistanceRecomp.exe directly (the D3D12 backend opens
//! its own window; there is no window host).

mod app_icon;
mod input_map;
mod mods;
mod play_shortcut;
mod recomplauncher;
mod report;
mod saves;
mod steam;
mod steam_art;
mod update;
mod vdf;
mod zipfile;

use input_map::{Bindings, InputEnv};
use saves::{read_sfo, BackupMeta, Side, Slot};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::UNIX_EPOCH;
use tauri::{AppHandle, Emitter, Manager, State};

const TITLE_ID: &str = "BCUS98107";
/// The game's executable name (a build from before 2026-10-22 has MyGameRecomp).
const GAME_EXE: &str = if cfg!(windows) { "ResistanceRecomp.exe" } else { "ResistanceRecomp" };
const LEGACY_EXE: &str = if cfg!(windows) { "MyGameRecomp.exe" } else { "MyGameRecomp" };
/// The window host, target `Resistance` of the EBOOT build (macOS; on
/// Windows the game opens its own window and there is no host).
const HOST_EXE: &str = "Resistance";
/// Play goes through the window host (macOS) rather than starting the EBOOT
/// build directly (Windows).
const USES_HOST: bool = !cfg!(windows);

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// What the player can change. Each field maps onto one of the environment
/// variables the port reads (see `launch_env`) or onto how Play starts it.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Launcher UI language: "pt-BR" or "en".
    pub ui_lang: String,
    /// The port checkout whose builds Play runs: it holds build/BCUS98107/
    /// and game/BCUS98107/ (this repository after tools/rebuild.sh, or the
    /// toolkit checkout the port was developed in). Empty = found on its own.
    pub port_dir: String,
    /// The disc: the folder holding PS3_GAME/ (a mounted image or an
    /// extracted copy). Empty = the first mounted BCUS98107 under /Volumes.
    pub disc_path: String,
    /// The disc image (.iso) to attach when the disc is not mounted.
    pub disc_image: String,
    /// Attach `disc_image` on Play when no disc is mounted.
    pub auto_mount: bool,
    /// Where saves, the install cache and last.log live (PS3_HDD0_ROOT,
    /// PS3_HDD1_ROOT, the cwd). Empty = ~/Library/Application Support/
    /// resistance-recomp, as tools/play.sh.
    pub play_dir: String,
    /// The campaign game.self: "auto" (v1.7 when its build is there) or
    /// "100" (RES_GAME_VERSION=100, always 01.00).
    pub campaign: String,
    /// PS3_LANG: the console language; empty = the Mac's.
    pub game_lang: String,
    /// The window host's --size: "<w>x<h>"; empty = 1280x720.
    pub window_size: String,
    /// PS3_WINDOW_MODE: "windowed" (the default), "fullscreen" (macOS's own)
    /// or "borderless".
    pub window_mode: String,
    /// PS3_DISPLAY: the screen's name (or its 1-based number when two share
    /// a name); empty = the main screen.
    pub display: String,
    /// PS3_VSYNC=0 when off.
    pub vsync: bool,
    /// PS3_UPSCALE: "" (a plain scale) or "metalfx" (macOS 13+).
    pub upscale: String,
    /// PS3_RES_SCALE: the internal resolution as a multiple of 1280x720,
    /// "" (native), "1.5" (1080p), "2" (1440p) or "3" (4K). Experimental.
    pub res_scale: String,
    /// The player's own ICON0.PNG in the Dock instead of the placeholder.
    pub game_icon: bool,
    /// PS3_AA: "" (off), "fxaa" or "smaa".
    pub aa: String,
    /// PS3_SHARPEN: 0 (off) to 1.
    pub sharpen: f32,
    /// PS3_ANISO: 0 (off), 2, 4, 8 or 16.
    pub aniso: u32,
    /// PS3_DISPLAY_GAMMA when set; otherwise the game's own brightness.
    pub gamma_override: bool,
    pub gamma: f32,
    /// PS3_VIBRANCE: -1 (muted) .. 0 (off) .. 1 (vivid).
    pub vibrance: f32,
    /// PS3_CONTRAST: 0.5 .. 1 (off) .. 1.5.
    pub contrast: f32,
    /// PS3_VIGNETTE and PS3_GRAIN: 0 (off) .. 1.
    pub vignette: f32,
    pub grain: f32,
    /// PS3_LUT: the player's colour LUT (ReShade's PNG layout), and how much
    /// of it (PS3_LUT_STRENGTH, 0..1).
    pub lut_path: String,
    pub lut_strength: f32,
    /// PS3_MODS: the texture packs in use (folders in <data>/mods), first
    /// wins; empty = none.
    pub mods: Vec<String>,
    /// PS3_MODS_DUMP=<data>/texture-dump: the game writes every texture it
    /// uploads there, named as a pack names it.
    pub mods_dump: bool,
    /// PS3_INPUT_MODE: "both" (the default), "controller" or "keyboard".
    pub input_mode: String,
    /// PS3_MOUSE, PS3_MOUSE_SENS, PS3_MOUSE_INVERT_Y, PS3_MOUSE_INVERT_X,
    /// PS3_MOUSE_DEADZONE (the runtime's defaults: on, 1.0, off, off, 30).
    pub mouse: bool,
    pub mouse_sens: f32,
    pub mouse_invert_y: bool,
    pub mouse_invert_x: bool,
    pub mouse_deadzone: u32,
    /// PS3_MOUSE_CAPTURE: a click captures the cursor (Esc/Cmd releases it).
    pub mouse_capture: bool,
    /// RES_MOUSE_LOOK=1 when on (experimental, off by default): the mouse
    /// turns the campaign's view by its own motion instead of driving the
    /// right stick (port/mouselook.cpp).
    pub mouse_direct: bool,
    /// PS3_RUMBLE=0 when off; PS3_RUMBLE_STRENGTH in percent (0-200).
    pub rumble: bool,
    pub rumble_strength: u32,
    /// PS3_PAD_ADAPTIVE=0 when off: a DualSense's triggers get a feel (a gun
    /// click on R2, light resistance on L2, R2 buzzing while firing).
    pub adaptive_triggers: bool,
    /// PS3_AUDIO_VOLUME (0-100, 100 = the default), PS3_AUDIO_MUTE=1 when
    /// muted, PS3_AUDIO_DEVICE (the output whose name contains it; empty =
    /// the system's default; macOS/Linux).
    pub audio_volume: u32,
    pub audio_mute: bool,
    pub audio_device: String,
    /// The port's optional game patches (port/patches.cpp), offered only when
    /// the build has them. "60 fps" is `unlock_fps`: it passes both
    /// RES_UNLOCK_FPS=1 (campaign frame limiter off) and RES_MENU_60FPS=1
    /// (menus and multiplayer). `menu_60fps` is the separate menus switch of
    /// earlier versions: a saved one still turns both on, and the window
    /// keeps the two equal. RES_FOV is the field-of-view factor (empty = the
    /// game's 1.0).
    pub unlock_fps: bool,
    pub menu_60fps: bool,
    pub fov: String,
    /// The online ID "Play online" signs in with (PS3_NP_ONLINE_ID): 3-16
    /// letters, digits, '-' or '_', starting with a letter. Empty = not set.
    pub online_id: String,
    /// Voice chat online: the computer's microphone as the PS3 headset
    /// (PS3_MIC=1, Play online only). Off by default.
    pub voice_chat: bool,
    /// PS3RECOMP_METAL_FPS=1: frame rate and frame times in the window title.
    pub show_fps: bool,
    /// PS3RECOMP_ENG_VCACHE=0 / PS3RECOMP_ENG_VCACHE_SPLIT=0 when off.
    pub vertex_cache: bool,
    pub vertex_cache_split: bool,
    /// PS3RECOMP_METAL_BIND_ALL=1 when off.
    pub bind_changed_only: bool,
    /// PS3_SHADER_CACHE=<data>/shader-cache when on.
    pub shader_cache: bool,
    /// PS3_SHADER_ASYNC=1 (opt-in).
    pub shader_async: bool,
    /// PS3_FRAMECLOCK_US: the host frame loop's tick (1000 = the default).
    pub frame_clock_us: u32,
    /// Run under `caffeinate -i` so the Mac does not sleep mid-game.
    pub keep_awake: bool,
    /// PS3RECOMP_FFMPEG=0 when off (no movies).
    pub video: bool,
    /// PS3RECOMP_FFMPEG_AVCODEC / _AVUTIL; empty = Homebrew's.
    pub ffmpeg_avcodec: String,
    pub ffmpeg_avutil: String,
    /// PS3_VERBOSE=0: the runtime's per-event lines stay out of last.log.
    pub quiet_log: bool,
    /// Extra KEY=VALUE lines (diagnostics), one per line; '#' starts a comment.
    pub extra_env: String,
    /// The rebuild's steps and JOBS (Advanced > Install / rebuild).
    pub rebuild_steps: Vec<String>,
    pub jobs: u32,
    /// Automatic backups of the saves made before each "Play" that are kept.
    pub save_backups_keep: u32,
    /// Look for a new launcher version and new port sources on start.
    pub auto_update_check: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            ui_lang: "pt-BR".into(),
            port_dir: String::new(),
            disc_path: String::new(),
            disc_image: String::new(),
            auto_mount: true,
            play_dir: String::new(),
            campaign: "auto".into(),
            game_lang: String::new(),
            window_size: String::new(),
            window_mode: "windowed".into(),
            display: String::new(),
            vsync: true,
            upscale: String::new(),
            res_scale: String::new(),
            game_icon: false,
            aa: String::new(),
            sharpen: 0.0,
            aniso: 0,
            gamma_override: false,
            gamma: 1.0,
            vibrance: 0.0,
            contrast: 1.0,
            vignette: 0.0,
            grain: 0.0,
            lut_path: String::new(),
            lut_strength: 1.0,
            mods: Vec::new(),
            mods_dump: false,
            input_mode: "both".into(),
            mouse: true,
            mouse_sens: 1.0,
            mouse_invert_y: false,
            mouse_invert_x: false,
            mouse_deadzone: 30,
            mouse_capture: true,
            mouse_direct: false,
            rumble: true,
            rumble_strength: 100,
            adaptive_triggers: true,
            audio_volume: 100,
            audio_mute: false,
            audio_device: String::new(),
            unlock_fps: false,
            menu_60fps: false,
            fov: String::new(),
            online_id: String::new(),
            voice_chat: false,
            show_fps: true,
            vertex_cache: true,
            vertex_cache_split: true,
            bind_changed_only: true,
            shader_cache: true,
            shader_async: false,
            frame_clock_us: 1000,
            keep_awake: true,
            video: true,
            ffmpeg_avcodec: String::new(),
            ffmpeg_avutil: String::new(),
            quiet_log: true,
            extra_env: String::new(),
            rebuild_steps: Vec::new(),
            jobs: 0,
            save_backups_keep: 10,
            auto_update_check: true,
        }
    }
}

fn home() -> PathBuf {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(var).or_else(|| std::env::var_os("HOME")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

/// %APPDATA% (Windows).
fn appdata() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| home().join("AppData").join("Roaming"))
}

/// The launcher's own folder: launcher.json, backups, reports, the shader
/// cache. Also the default play folder (tools/play.sh's RES_DATA).
/// RES_LAUNCHER_DATA overrides it (tests, a second setup beside the player's).
fn data_root() -> PathBuf {
    if let Some(d) = std::env::var_os("RES_LAUNCHER_DATA").filter(|v| !v.is_empty()) {
        return PathBuf::from(d);
    }
    if cfg!(target_os = "macos") {
        home().join("Library/Application Support/resistance-recomp")
    } else if cfg!(windows) {
        appdata().join("resistance-recomp")
    } else {
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".local/share")).join("resistance-recomp")
    }
}

fn settings_path() -> PathBuf {
    data_root().join("launcher.json")
}

/// The build tools and libraries tools/bootstrap_deps.sh installs (CMake,
/// Ninja, Python, SDL2, glslang, SPIRV-Cross, FFmpeg): RES_DEPS, else
/// <data>/deps. Preferred over Homebrew's whenever they are there.
fn deps_dir() -> PathBuf {
    std::env::var_os("RES_DEPS").filter(|v| !v.is_empty()).map(PathBuf::from).unwrap_or_else(|| data_root().join("deps"))
}

/// tools/bootstrap_deps.sh finished there (it writes manifest.txt last).
fn deps_ready(d: &Path) -> bool {
    d.join("manifest.txt").is_file() && d.join("include/libavcodec/avcodec.h").is_file()
}

/// Where "Install the game" puts the port's sources when this Mac has no
/// checkout of them (a player who only downloaded the app).
fn src_dir() -> PathBuf {
    data_root().join("src")
}

/// The last install's output, as the window showed it.
fn install_log_path() -> PathBuf {
    data_root().join("install.log")
}

/// Held (flock) while an install runs, the window's or a headless
/// `--prepare`'s (recomplauncher.rs), so two never run at once; released
/// when the holder exits, however it exits.
fn install_lock_path() -> PathBuf {
    data_root().join("install.lock")
}

/// The install lock, taken: Ok(None) when another install holds it.
fn install_lock() -> Result<Option<fs::File>, String> {
    let p = install_lock_path();
    if let Some(d) = p.parent() {
        fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let f = fs::OpenOptions::new().create(true).truncate(false).write(true).open(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    match f.try_lock() {
        Ok(()) => Ok(Some(f)),
        Err(fs::TryLockError::WouldBlock) => Ok(None),
        Err(fs::TryLockError::Error(e)) => Err(format!("{}: {e}", p.display())),
    }
}

/// The port's repository ("Install the game" clones it; RES_SOURCES_URL
/// overrides it, e.g. with a fork or a mirror).
const SOURCES_URL: &str = "https://github.com/erikomis/resistance-recomp.git";

fn sources_url() -> String {
    std::env::var("RES_SOURCES_URL").ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| SOURCES_URL.into())
}

/// The free space a full install asks for: it leaves about 1.5 GB (measured:
/// the dependencies 0.5 GB with their downloads, the toolkit and the venv
/// 0.07 GB, the disc's executables and ELFs 0.15 GB, the analysis and the lift
/// 0.24 GB, the two builds 0.45 GB), with room for the build trees on the way
/// and the play folder's caches; tools/install.sh asks for the same.
const INSTALL_NEED_GB: u64 = 4;

fn reports_dir() -> PathBuf {
    data_root().join("reports")
}

/// The controls file (PS3_INPUT_MAP), next to launcher.json. It exists once
/// the Controls page has changed a binding; until then the game's built-in
/// layout applies and Play passes nothing.
fn input_map_path() -> PathBuf {
    data_root().join("input.toml")
}

/// Where texture packs live (PS3_MODS_DIR): <data>/mods.
fn mods_dir() -> PathBuf {
    data_root().join("mods")
}

/// Where "Dump textures" makes the game write them (PS3_MODS_DUMP).
fn mods_dump_dir() -> PathBuf {
    data_root().join("texture-dump")
}

/// The controls file as Play passes it (see `launch_env`).
fn input_env() -> InputEnv {
    let p = input_map_path();
    match fs::read_to_string(&p) {
        Ok(text) => InputEnv { path: Some(s(&p)), has_mouse_section: input_map::parse(&text).has_mouse_section },
        Err(_) => InputEnv::default(),
    }
}

fn rpcs3_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        home().join("Library/Application Support/rpcs3")
    } else if cfg!(windows) {
        // RPCS3 for Windows keeps dev_hdd0 next to rpcs3.exe
        find_rpcs3().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_else(|| home().join("rpcs3"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".config")).join("rpcs3")
    }
}

fn load_settings_file() -> Settings {
    fs::read_to_string(settings_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn write_settings(s: &Settings) -> Result<(), String> {
    let p = settings_path();
    if let Some(d) = p.parent() {
        fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let tmp = p.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(s).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &p).map_err(|e| e.to_string())
}

/// The play folder: the setting, else the data root (as tools/play.sh).
fn play_dir(s: &Settings) -> PathBuf {
    let p = s.play_dir.trim();
    if p.is_empty() {
        data_root()
    } else {
        PathBuf::from(p)
    }
}

fn parse_env_lines(text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let l = l.strip_prefix("export ").unwrap_or(l);
            let (k, v) = l.split_once('=')?;
            let k = k.trim();
            if k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return None;
            }
            Some((k.to_string(), v.trim().trim_matches('"').to_string()))
        })
        .collect()
}

fn extra_var(s: &Settings, key: &str) -> Option<String> {
    parse_env_lines(&s.extra_env).into_iter().rev().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn mtime(p: &Path) -> u64 {
    fs::metadata(p).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs())
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        s.push(T[(n >> 18) as usize & 63] as char);
        s.push(T[(n >> 12) as usize & 63] as char);
        s.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        s.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    s
}

fn data_url(path: &Path, mime: &str) -> Option<String> {
    let mut buf = Vec::new();
    fs::File::open(path).ok()?.read_to_end(&mut buf).ok()?;
    Some(format!("data:{mime};base64,{}", base64(&buf)))
}

fn s(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

// ---------------------------------------------------------------------------
// The disc
// ---------------------------------------------------------------------------

/// A folder that holds a PS3 disc's PS3_GAME/ (or is PS3_GAME itself).
fn disc_root_of(p: &Path) -> Option<PathBuf> {
    if p.join("PS3_GAME/PARAM.SFO").is_file() {
        Some(p.to_path_buf())
    } else if p.file_name().is_some_and(|n| n == "PS3_GAME") && p.join("PARAM.SFO").is_file() {
        p.parent().map(Path::to_path_buf)
    } else {
        None
    }
}

fn disc_title_id(root: &Path) -> Option<String> {
    read_sfo(&root.join("PS3_GAME/PARAM.SFO")).and_then(|v| v.get("TITLE_ID").cloned())
}

/// The places a disc can be mounted: the volumes under /Volumes (macOS), or
/// the drive roots A:\ to Z:\ (Windows).
fn volume_roots() -> Vec<PathBuf> {
    if cfg!(windows) {
        (b'A'..=b'Z').map(|c| PathBuf::from(format!("{}:\\", c as char))).collect()
    } else {
        fs::read_dir("/Volumes").into_iter().flatten().flatten().map(|e| e.path()).collect()
    }
}

/// Mounted volumes that are this game's disc (as tools/env.sh finds them).
fn mounted_discs() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = volume_roots()
        .into_iter()
        .filter(|p| p.join("PS3_GAME/USRDIR/game.self").is_file() && disc_title_id(p).as_deref() == Some(TITLE_ID))
        .collect();
    v.sort();
    v
}

/// The disc Play uses: the setting when it points at one, else the first
/// mounted BCUS98107.
fn find_disc(s: &Settings) -> Option<PathBuf> {
    let p = s.disc_path.trim();
    if !p.is_empty() {
        if let Some(r) = disc_root_of(Path::new(p)) {
            return Some(r);
        }
    }
    mounted_discs().into_iter().next()
}

/// The string values of `key` in an XML plist, in order.
fn plist_strings(xml: &str, key: &str) -> Vec<String> {
    let tag = format!("<key>{key}</key>");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(i) = rest.find(&tag) {
        rest = &rest[i + tag.len()..];
        let t = rest.trim_start();
        if let Some(v) = t.strip_prefix("<string>").and_then(|v| v.split_once("</string>")).map(|(v, _)| v) {
            out.push(v.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">"));
        }
    }
    out
}

/// Disc images attached now: (image path, its mount points).
fn attached_images() -> Vec<(String, Vec<String>)> {
    if !cfg!(target_os = "macos") {
        return Vec::new();
    }
    let Some(xml) = command_stdout(Command::new("/usr/bin/hdiutil").args(["info", "-plist"])) else { return Vec::new() };
    // one <key>image-path</key> per attached image; its mount points follow
    // before the next one
    let tag = "<key>image-path</key>";
    let mut out = Vec::new();
    let parts: Vec<&str> = xml.split(tag).collect();
    for part in parts.iter().skip(1) {
        let chunk = format!("{tag}{part}");
        let image = plist_strings(&chunk, "image-path").into_iter().next().unwrap_or_default();
        out.push((image, plist_strings(&chunk, "mount-point")));
    }
    out
}

fn image_of(mount: &Path) -> Option<String> {
    let m = s(mount);
    attached_images().into_iter().find(|(_, mps)| mps.iter().any(|p| *p == m)).map(|(i, _)| i)
}

/// The drive letter of a drive root ("E:\\" or "E:"), Windows.
fn drive_letter(root: &Path) -> Option<char> {
    let t = s(root);
    let b = t.as_bytes();
    let ok = matches!(b.len(), 2 | 3) && b[0].is_ascii_alphabetic() && b[1] == b':' && (b.len() == 2 || b[2] == b'\\' || b[2] == b'/');
    ok.then(|| (b[0] as char).to_ascii_uppercase())
}

/// Mount-DiskImage the image in $env:RES_IMAGE (read only; reused when it is
/// attached already) and print the drive letter it got. A fixed script: the
/// path reaches PowerShell only through the environment, never as source.
const PS_ATTACH: &str = "$ErrorActionPreference = 'Stop'
$p = $env:RES_IMAGE
$i = Get-DiskImage -ImagePath $p
if (-not $i.Attached) { $i = Mount-DiskImage -ImagePath $p -Access ReadOnly -PassThru }
$l = $null
for ($n = 0; $n -lt 40 -and -not $l; $n++) {
  $l = ($i | Get-Volume -ErrorAction SilentlyContinue).DriveLetter
  if (-not $l) { Start-Sleep -Milliseconds 250; $i = Get-DiskImage -ImagePath $p }
}
Write-Output ([string]$l)";

/// Dismount-DiskImage the image behind the drive letter in $env:RES_DRIVE.
const PS_DETACH: &str = "$ErrorActionPreference = 'Stop'
Get-Volume -DriveLetter $env:RES_DRIVE | Get-DiskImage | Dismount-DiskImage | Out-Null";

/// -EncodedCommand's form of a script: base64 of its UTF-16LE text.
fn ps_encoded(script: &str) -> String {
    base64(&script.encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<u8>>())
}

/// The PowerShell command line for a fixed `script` (-EncodedCommand, so no
/// command-line quoting is involved), and the environment that carries its
/// inputs (paths, drive letters) out of band.
fn powershell_cmd(script: &'static str, inputs: &[(&str, &std::ffi::OsStr)]) -> Command {
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", &ps_encoded(script)]).stdin(Stdio::null());
    for (k, v) in inputs {
        cmd.env(k, v);
    }
    no_console(&mut cmd);
    cmd
}

/// Run a fixed PowerShell script (Windows) and return its output.
fn powershell(script: &'static str, inputs: &[(&str, &std::ffi::OsStr)]) -> Result<String, String> {
    let out = powershell_cmd(script, inputs).output().map_err(|e| format!("powershell: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Windows: attach the image and return the drive root it got.
fn attach_image_windows(image: &Path) -> Result<PathBuf, String> {
    let out = powershell(PS_ATTACH, &[("RES_IMAGE", image.as_os_str())]).map_err(|e| format!("Mount-DiskImage: {e}"))?;
    let letter = out.lines().last().unwrap_or("").trim().chars().next().filter(char::is_ascii_alphabetic).ok_or_else(|| format!("{}: mounted, but Windows gave it no drive letter", image.display()))?;
    let root = PathBuf::from(format!("{letter}:\\"));
    disc_root_of(&root).ok_or_else(|| format!("{}: mounted as {letter}:, but it holds no PS3_GAME folder (not a PS3 disc image?)", image.display()))
}

/// Windows: Dismount-DiskImage the image behind a drive letter.
fn detach_windows(letter: char) -> Result<(), String> {
    let l = letter.to_string();
    powershell(PS_DETACH, &[("RES_DRIVE", std::ffi::OsStr::new(&l))])
        .map(|_| ())
        .map_err(|e| format!("{letter}: is not a mounted disc image, or Windows could not eject it ({e})"))
}

/// Attach the image (read-only) and return the disc's mount point:
/// `hdiutil attach` on macOS, Mount-DiskImage on Windows.
fn attach_image(image: &Path) -> Result<PathBuf, String> {
    if !image.is_file() {
        return Err(format!("{}: no such file", image.display()));
    }
    if cfg!(windows) {
        return attach_image_windows(image);
    }
    if !cfg!(target_os = "macos") {
        return Err("mounting a disc image is macOS and Windows only".into());
    }
    // already attached: its mount point
    let img = s(image);
    if let Some((_, mps)) = attached_images().into_iter().find(|(i, _)| *i == img) {
        if let Some(r) = mps.iter().map(PathBuf::from).find_map(|p| disc_root_of(&p)) {
            return Ok(r);
        }
    }
    let out = Command::new("/usr/bin/hdiutil")
        .args(["attach", "-readonly", "-plist"])
        .arg(image)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("hdiutil: {e}"))?;
    if !out.status.success() {
        return Err(format!("hdiutil attach: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let xml = String::from_utf8_lossy(&out.stdout);
    plist_strings(&xml, "mount-point")
        .into_iter()
        .map(PathBuf::from)
        .find_map(|p| disc_root_of(&p))
        .ok_or_else(|| format!("{}: mounted, but it holds no PS3_GAME folder (not a PS3 disc image?)", image.display()))
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscInfo {
    root: Option<String>,
    title: String,
    title_id: String,
    app_ver: String,
    /// The image it is mounted from, when it is one.
    image: Option<String>,
    /// The disc is this game (BCUS98107).
    ours: bool,
    /// Other BCUS98107 volumes mounted now.
    mounted: Vec<String>,
}

fn disc_info(s: &Settings) -> DiscInfo {
    let mut d = DiscInfo { mounted: mounted_discs().iter().map(|p| self::s(p)).collect(), ..Default::default() };
    if let Some(r) = find_disc(s) {
        let v = read_sfo(&r.join("PS3_GAME/PARAM.SFO")).unwrap_or_default();
        d.title = v.get("TITLE").cloned().unwrap_or_default();
        d.title_id = v.get("TITLE_ID").cloned().unwrap_or_default();
        d.app_ver = v.get("APP_VER").or(v.get("VERSION")).cloned().unwrap_or_default();
        d.ours = d.title_id == TITLE_ID;
        if cfg!(windows) {
            // the image this launcher mounted there (no PowerShell per status poll)
            let same = |a: &Path, b: &Path| self::s(a).trim_end_matches(['\\', '/']).eq_ignore_ascii_case(self::s(b).trim_end_matches(['\\', '/']));
            if drive_letter(&r).is_some() && !s.disc_image.trim().is_empty() && same(Path::new(s.disc_path.trim()), &r) {
                d.image = Some(s.disc_image.trim().to_string());
            }
        } else if r.starts_with("/Volumes") {
            d.image = image_of(&r);
        }
        d.root = Some(self::s(&r));
    }
    d
}

/// The disc's ICON0.PNG and PIC1.PNG (the player's own; nothing ships).
fn disc_art(s: &Settings) -> (Option<PathBuf>, Option<PathBuf>) {
    match find_disc(s) {
        Some(r) => {
            let g = r.join("PS3_GAME");
            (Some(g.join("ICON0.PNG")).filter(|p| p.is_file()), Some(g.join("PIC1.PNG")).filter(|p| p.is_file()))
        }
        None => (None, None),
    }
}

// ---------------------------------------------------------------------------
// The builds
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Builds {
    root: String,
    /// The window host, the EBOOT build and the campaign builds.
    host: Option<String>,
    eboot: Option<String>,
    game: Option<String>,
    game_v17: Option<String>,
    eboot_elf: Option<String>,
    game_elf: Option<String>,
    game_v17_elf: Option<String>,
    /// The v1.7 menus (tools/v17/regen_eboot_v17.sh), which online play runs.
    eboot_v17: Option<String>,
    eboot_v17_elf: Option<String>,
    /// Everything Play needs is there (host, EBOOT build and ELF, 01.00
    /// campaign build and ELF).
    ok: bool,
    /// The v1.7 campaign can run (its build and ELF).
    v17: bool,
    /// The builds carry the old executable name.
    legacy: bool,
    missing: Vec<String>,
    /// tools/rebuild.sh, when this is the port's own repository.
    rebuild_sh: Option<String>,
    /// When the EBOOT build was made (its mtime).
    built_at: u64,
}

fn exe_in(dir: &Path) -> (Option<PathBuf>, bool) {
    let p = dir.join(GAME_EXE);
    if p.is_file() {
        return (Some(p), false);
    }
    let old = dir.join(LEGACY_EXE);
    if old.is_file() {
        return (Some(old), true);
    }
    (None, false)
}

fn resolve_builds(root: &Path) -> Builds {
    let b = root.join("build");
    let g = root.join("game").join(TITLE_ID);
    let (eboot, l1) = exe_in(&b.join(TITLE_ID));
    let (game, l2) = exe_in(&b.join(format!("{TITLE_ID}_game")));
    let (game_v17, _) = exe_in(&b.join(format!("{TITLE_ID}_game_v17")));
    let (eboot_v17, _) = exe_in(&b.join(format!("{TITLE_ID}_eboot_v17")));
    let host = Some(b.join(TITLE_ID).join(HOST_EXE)).filter(|p| USES_HOST && p.is_file());
    let file = |p: PathBuf| Some(p).filter(|p| p.is_file());
    let eboot_elf = file(g.join("EBOOT.elf"));
    let game_elf = file(g.join("modules/game.elf"));
    let game_v17_elf = file(g.join("modules_v17/game.elf"));
    let eboot_v17_elf = file(g.join("EBOOT_v17.elf"));
    let mut missing = Vec::new();
    for (what, v) in [
        (format!("build/{TITLE_ID}/{HOST_EXE}"), host.is_some() || !USES_HOST),
        (format!("build/{TITLE_ID}/{GAME_EXE}"), eboot.is_some()),
        (format!("build/{TITLE_ID}_game/{GAME_EXE}"), game.is_some()),
        (format!("game/{TITLE_ID}/EBOOT.elf"), eboot_elf.is_some()),
        (format!("game/{TITLE_ID}/modules/game.elf"), game_elf.is_some()),
    ] {
        if !v {
            missing.push(what);
        }
    }
    let rebuild_sh = file(root.join("tools/rebuild.sh"));
    let o = |p: &Option<PathBuf>| p.as_deref().map(s);
    Builds {
        root: s(root),
        built_at: eboot.as_deref().map(mtime).unwrap_or(0),
        ok: missing.is_empty(),
        v17: game_v17.is_some() && game_v17_elf.is_some(),
        legacy: l1 || l2,
        host: o(&host),
        eboot: o(&eboot),
        game: o(&game),
        game_v17: o(&game_v17),
        eboot_elf: o(&eboot_elf),
        game_elf: o(&game_elf),
        game_v17_elf: o(&game_v17_elf),
        eboot_v17: o(&eboot_v17).filter(|_| eboot_v17_elf.is_some()),
        eboot_v17_elf: o(&eboot_v17_elf).filter(|_| eboot_v17.is_some()),
        missing,
        rebuild_sh: o(&rebuild_sh),
    }
}

/// The checkout this launcher was built from (development builds, and an
/// .app built inside the repository by tools/rebuild.sh launcher).
fn own_repo() -> Option<PathBuf> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    p.join("tools/rebuild.sh").is_file().then(|| p.canonicalize().unwrap_or(p))
}

/// Port folders found on this Mac: RES_ROOT, the repository the launcher
/// comes from, then a folder one level under the usual places that has the
/// EBOOT build. Complete builds first, the newest build first.
fn find_port_dirs_now() -> Vec<Builds> {
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Some(r) = std::env::var_os("RES_ROOT") {
        cands.push(PathBuf::from(r));
    }
    cands.extend(own_repo());
    // "Install the game"'s own copy of the sources
    if src_dir().join("tools/rebuild.sh").is_file() {
        cands.push(src_dir());
    }
    for base in ["Desktop", "Developer", "Documents", "Projects", "src", "code", "git", ""] {
        let dir = home().join(base);
        let Ok(it) = fs::read_dir(&dir) else { continue };
        for e in it.flatten() {
            let p = e.path();
            if p.join("build").join(TITLE_ID).is_dir() && !e.file_name().to_string_lossy().starts_with('.') {
                cands.push(p);
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    let mut v: Vec<Builds> = cands
        .into_iter()
        .filter(|p| p.is_dir())
        .map(|p| p.canonicalize().unwrap_or(p))
        .filter(|p| seen.insert(p.clone()))
        .map(|p| resolve_builds(&p))
        .collect();
    v.sort_by(|a, b| b.ok.cmp(&a.ok).then(b.built_at.cmp(&a.built_at)));
    v
}

/// The port folder in use: the setting, else the best one found.
fn port_root(s: &Settings) -> Option<PathBuf> {
    let p = s.port_dir.trim();
    if !p.is_empty() {
        return Some(PathBuf::from(p));
    }
    find_port_dirs_now().into_iter().find(|b| b.ok || b.rebuild_sh.is_some()).map(|b| PathBuf::from(b.root))
}

// ---------------------------------------------------------------------------
// The play folder
// ---------------------------------------------------------------------------

fn game_data_dir(play: &Path) -> PathBuf {
    play.join("dev_hdd0/game").join(TITLE_ID)
}

/// The v1.7 update's patch_NN.bin files in the play folder's USRDIR/data:
/// the 01.00 EBOOT cannot run with the update's archives mounted (it crashes
/// loading the menu fonts with patch_04/patch_06, and cannot shut FIOS down
/// at the exitspawn with the others). It mounts a patch_NN.psarc only when
/// the patch_NN.bin is beside it, so it is enough that no .bin is there.
fn update_files(play: &Path) -> Vec<PathBuf> {
    let dir = game_data_dir(play).join("USRDIR/data");
    let mut v: Vec<PathBuf> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let n = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            n.starts_with("patch_") && n.ends_with(".bin") && p.is_file()
        })
        .collect();
    v.sort();
    v
}

/// The first boot's "initial setup" (the HDD install) has not run here yet.
fn first_boot(play: &Path) -> bool {
    !game_data_dir(play).join("USRDIR/cache").is_dir()
}

fn save_locations(s: &Settings) -> saves::Locations {
    let play = play_dir(s);
    let gamedata = extra_var(s, "PS3_GAMEDATA_ROOT").map(PathBuf::from).unwrap_or_else(|| play.join("gamedata"));
    let hdd0 = extra_var(s, "PS3_HDD0_ROOT").map(PathBuf::from).unwrap_or_else(|| play.join("dev_hdd0"));
    saves::Locations {
        port_savedata: gamedata.join("dev_hdd0/home").join(saves::PORT_USER).join("savedata"),
        port_usrdir: hdd0.join("game").join(TITLE_ID).join("USRDIR"),
        rpcs3_hdd0: rpcs3_dir().join("dev_hdd0"),
        backups: data_root().join("save-backups"),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveInfo {
    name: String,
    title: String,
    subtitle: String,
    modified: u64,
    folder: String,
}

fn last_save(s: &Settings) -> Option<SaveInfo> {
    saves::list_slots(&save_locations(s), Side::Port, "", false)
        .into_iter()
        .max_by_key(|x| x.modified)
        .map(|x| SaveInfo { name: x.name, title: x.title, subtitle: x.subtitle, modified: x.modified, folder: x.path })
}

fn shader_cache_dir(s: &Settings) -> Option<PathBuf> {
    match extra_var(s, "PS3_SHADER_CACHE") {
        Some(v) if v.is_empty() || v == "0" => None,
        Some(v) => Some(PathBuf::from(v)),
        None => s.shader_cache.then(|| data_root().join("shader-cache")),
    }
}

// ---------------------------------------------------------------------------
// Processes
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct AppState {
    rebuild: Mutex<Option<u32>>,
    game: Mutex<Option<u32>>,
    setup: Mutex<Option<u32>>,
    /// The report folder of the game started in debug mode, and whether that
    /// game is still running.
    report: Mutex<Option<(PathBuf, bool)>>,
    /// Where the last game this launcher started writes its output.
    last_log: Mutex<Option<PathBuf>>,
    /// The launcher's Dock icon as last set (app_icon::key; "" = the bundle's).
    dock_icon: Mutex<String>,
    play_request: Mutex<bool>,
    /// `--play-online` without RecompLauncher: start online play as Home's
    /// Play online button does, once the window is up.
    online_request: Mutex<bool>,
    exit_with_game: Mutex<bool>,
    /// `--view <page>`: the page to open on (play_shortcut::start_view).
    start_view: Mutex<Option<String>>,
    /// build_features' cache.
    features: Mutex<FeatureCache>,
    /// An app update is being downloaded and installed (update.rs).
    updating: Mutex<bool>,
    /// The version an update put on disk this session (restart to finish).
    update_installed: Mutex<Option<String>>,
}

/// Play, the rebuild and the install wait while the app updates itself.
fn refuse_while_updating(state: &AppState) -> Result<(), String> {
    if *state.updating.lock().unwrap() {
        return Err("the launcher is installing its update: wait for it to finish".into());
    }
    Ok(())
}

/// A GUI app on macOS starts with a bare PATH; tools/bootstrap_deps.sh's
/// tools, then Homebrew's and the usual install places are added in front.
/// Windows: PATH as it is.
fn tool_path() -> std::ffi::OsString {
    let mut parts: Vec<PathBuf> = Vec::new();
    if cfg!(target_os = "macos") {
        let deps = deps_dir().join("bin");
        if deps.is_dir() {
            parts.push(deps);
        }
        parts.extend(["/opt/homebrew/bin", "/opt/homebrew/sbin", "/usr/local/bin"].map(PathBuf::from));
    }
    if let Some(p) = std::env::var_os("PATH") {
        parts.extend(std::env::split_paths(&p));
    }
    if cfg!(unix) {
        parts.extend(["/usr/bin", "/bin", "/usr/sbin", "/sbin"].map(PathBuf::from));
    }
    let mut seen = std::collections::HashSet::new();
    parts.retain(|p| seen.insert(p.clone()));
    std::env::join_paths(parts).unwrap_or_else(|_| std::env::var_os("PATH").unwrap_or_default())
}

/// The game builds and runs on macOS (Metal). Windows is let through as a
/// preview: the port's D3D12 build there is untested (Status.experimental).
fn host_supported() -> bool {
    cfg!(target_os = "macos") || cfg!(windows) || std::env::var_os("RES_ANY_HOST").is_some_and(|v| !v.is_empty())
}

fn require_supported_host() -> Result<(), String> {
    if host_supported() {
        Ok(())
    } else {
        Err(format!("the game runs on macOS only; this is {}", std::env::consts::OS))
    }
}

/// No console window for a child of this GUI app (Windows).
#[cfg(windows)]
pub(crate) fn no_console(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW)
}
#[cfg(not(windows))]
pub(crate) fn no_console(cmd: &mut Command) -> &mut Command {
    cmd
}

fn clt_installed() -> bool {
    if !cfg!(target_os = "macos") {
        return true;
    }
    Command::new("xcode-select")
        .arg("-p")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|o| o.status.success() && Path::new(String::from_utf8_lossy(&o.stdout).trim()).is_dir())
        .unwrap_or(false)
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    std::env::split_paths(&tool_path()).map(|d| d.join(&file)).find(|p| p.is_file())
}

fn find_brew() -> Option<PathBuf> {
    ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"].iter().map(PathBuf::from).find(|p| p.is_file())
}

fn brew_prefix() -> PathBuf {
    match find_brew() {
        Some(b) => b.parent().and_then(Path::parent).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("/opt/homebrew")),
        None => PathBuf::from("/opt/homebrew"),
    }
}

/// libavcodec / libavutil for the video and AC3 decoders: the settings, else
/// tools/bootstrap_deps.sh's, else Homebrew's (the unversioned link, or the
/// newest versioned one).
fn ffmpeg_libs(s: &Settings) -> (Option<PathBuf>, Option<PathBuf>) {
    if cfg!(windows) {
        // optional there: the paths set, else whatever the port finds itself
        let over = |v: &str| Some(PathBuf::from(v.trim())).filter(|_| !v.trim().is_empty());
        return (over(&s.ffmpeg_avcodec), over(&s.ffmpeg_avutil));
    }
    let libs = [deps_dir().join("lib"), brew_prefix().join("lib")];
    let pick = |over: &str, name: &str| -> Option<PathBuf> {
        if !over.trim().is_empty() {
            return Some(PathBuf::from(over.trim()));
        }
        libs.iter().find_map(|lib| {
            let plain = lib.join(format!("{name}.dylib"));
            if plain.exists() {
                return Some(plain);
            }
            let mut v: Vec<PathBuf> = fs::read_dir(lib)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with(&format!("{name}.")) && n.to_string_lossy().ends_with(".dylib")))
                .collect();
            v.sort();
            v.pop()
        })
    };
    (pick(&s.ffmpeg_avcodec, "libavcodec"), pick(&s.ffmpeg_avutil, "libavutil"))
}

#[cfg(unix)]
fn own_process_group(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0); // so a stop reaches what it started too
}
#[cfg(not(unix))]
fn own_process_group(cmd: &mut Command) {
    no_console(cmd);
}

/// Stop what this launcher started, with what it started in turn: SIGTERM
/// (SIGKILL when `force`) to the process group `pid` leads; on Windows
/// `taskkill /T` (a close request to its windows; /F when `force`).
fn kill_tree(pid: u32, force: bool) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill").arg(if force { "-KILL" } else { "-TERM" }).arg(format!("-{pid}")).status();
    }
    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/PID", &pid.to_string(), "/T"]);
        if force {
            cmd.arg("/F");
        }
        let _ = no_console(&mut cmd).stdout(Stdio::null()).stderr(Stdio::null()).status();
    }
    #[cfg(not(any(unix, windows)))]
    let _ = (pid, force);
}

fn command_stdout(cmd: &mut Command) -> Option<String> {
    let out = cmd.stderr(Stdio::null()).output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

/// A process with this exact name is running (the game started from a
/// terminal). Only looked at, never signalled.
#[cfg(not(windows))]
fn process_running(name: &str) -> bool {
    Command::new("pgrep").args(["-x", name]).stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
}
#[cfg(windows)]
fn process_running(name: &str) -> bool {
    let image = if name.to_ascii_lowercase().ends_with(".exe") { name.to_string() } else { format!("{name}.exe") };
    let mut cmd = Command::new("tasklist");
    cmd.args(["/FI", &format!("IMAGENAME eq {image}"), "/NH", "/FO", "CSV"]);
    no_console(&mut cmd)
        .stderr(Stdio::null())
        .output()
        .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).to_ascii_lowercase().contains(&format!("\"{}\"", image.to_ascii_lowercase())))
}

fn shell_quote(v: &str) -> String {
    if !v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric() || "-_./:,=+@".contains(c)) {
        v.to_string()
    } else {
        format!("'{}'", v.replace('\'', "'\\''"))
    }
}

// ---------------------------------------------------------------------------
// Launch
// ---------------------------------------------------------------------------

/// "<w>x<h>", as the window host's --size takes it.
fn valid_window_size(v: &str) -> bool {
    let mut it = v.splitn(2, 'x').map(|n| n.parse::<u32>().ok());
    matches!((it.next(), it.next()), (Some(Some(w)), Some(Some(h))) if (320..=8192).contains(&w) && (180..=8192).contains(&h))
}

/// PS3_RES_SCALE for the `res_scale` setting: a multiple above 1 and up to 4
/// (the runtime's range, in quarter steps), or None for native (also for
/// anything unparsable).
fn res_scale_value(v: &str) -> Option<String> {
    let x: f32 = v.trim().parse().ok()?;
    (x > 1.0 && x <= 4.0).then(|| format!("{}", (x * 4.0).round() / 4.0))
}

/// RES_FOV for the `fov` setting: a factor the patch accepts (0.1 .. 4) other
/// than the game's own 1.0, or None.
fn fov_value(v: &str) -> Option<String> {
    let x: f32 = v.trim().parse().ok()?;
    ((0.1..=4.0).contains(&x) && (x - 1.0).abs() > 1e-3).then(|| format!("{:.2}", x))
}

/// The environment Play gives the game, on top of the launcher's own: the
/// disc, the play folder's dev_hdd0/dev_hdd1, FFmpeg, and every setting that
/// differs from the port's default; the extra variables last (they win).
/// `input`: the controls file, when the Controls page has written one.
fn launch_env(s: &Settings, disc: &Path, play: &Path, input: &InputEnv) -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = Vec::new();
    let mut set = |k: &str, v: String| env.push((k.to_string(), v));
    set("PS3_VFS_ROOT", s_(disc));
    set("PS3_HDD0_ROOT", s_(&play.join("dev_hdd0")));
    set("PS3_HDD1_ROOT", s_(&play.join("dev_hdd1")));
    // The in-game menu's settings (F1): what the player changes there is saved
    // to this file and read again at the next start and across the
    // EBOOT -> game.self exec. The variables below still win over it.
    set("PS3_OVERLAY_INI", s_(&play.join("overlay.ini")));
    if s.video {
        let (codec, util) = ffmpeg_libs(s);
        if let Some(c) = codec {
            set("PS3RECOMP_FFMPEG_AVCODEC", s_(&c));
        }
        if let Some(u) = util {
            set("PS3RECOMP_FFMPEG_AVUTIL", s_(&u));
        }
    } else {
        set("PS3RECOMP_FFMPEG", "0".into());
    }
    if s.campaign == "100" {
        set("RES_GAME_VERSION", "100".into());
    }
    let lang = s.game_lang.trim();
    if !lang.is_empty() && lang.len() <= 8 && lang.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        set("PS3_LANG", lang.to_string());
    }
    if s.aa == "fxaa" || s.aa == "smaa" {
        set("PS3_AA", s.aa.clone());
    }
    if s.sharpen >= 0.005 {
        set("PS3_SHARPEN", format!("{:.2}", s.sharpen.min(1.0)));
    }
    if s.aniso > 1 {
        set("PS3_ANISO", s.aniso.min(16).to_string());
    }
    if s.gamma_override {
        set("PS3_DISPLAY_GAMMA", format!("{:.2}", s.gamma.clamp(0.5, 1.5)));
    }
    // The present pass's filters: only what differs from the game's picture.
    if s.vibrance.abs() >= 0.005 {
        set("PS3_VIBRANCE", format!("{:.2}", s.vibrance.clamp(-1.0, 1.0)));
    }
    if (s.contrast - 1.0).abs() >= 0.005 && s.contrast > 0.0 {
        set("PS3_CONTRAST", format!("{:.2}", s.contrast.clamp(0.5, 1.5)));
    }
    if s.vignette >= 0.005 {
        set("PS3_VIGNETTE", format!("{:.2}", s.vignette.min(1.0)));
    }
    if s.grain >= 0.005 {
        set("PS3_GRAIN", format!("{:.2}", s.grain.min(1.0)));
    }
    let lut = s.lut_path.trim();
    if !lut.is_empty() {
        set("PS3_LUT", lut.to_string());
        if s.lut_strength < 0.995 {
            set("PS3_LUT_STRENGTH", format!("{:.2}", s.lut_strength.clamp(0.0, 1.0)));
        }
    }
    if let Some(k) = res_scale_value(&s.res_scale) {
        set("PS3_RES_SCALE", k);
    }
    // Texture packs: the folder and the enabled ones, first wins; the dump.
    if let Some(list) = mods::env_list(&s.mods) {
        set("PS3_MODS_DIR", s_(&mods_dir()));
        set("PS3_MODS", list);
    }
    if s.mods_dump {
        set("PS3_MODS_DUMP", s_(&mods_dump_dir()));
    }
    // The window (tools/host_window). The size stays the host's --size.
    if matches!(s.window_mode.as_str(), "fullscreen" | "borderless") {
        set("PS3_WINDOW_MODE", s.window_mode.clone());
    }
    let display = s.display.trim();
    if !display.is_empty() && !display.contains(['\n', '\r']) {
        set("PS3_DISPLAY", display.to_string());
    }
    if !s.vsync {
        set("PS3_VSYNC", "0".into());
    }
    if s.upscale == "metalfx" {
        set("PS3_UPSCALE", "metalfx".into());
    }
    // Input: the controls file, then the mouse and rumble settings. The
    // variables win over the file's [mouse]; when the file has one (a hand
    // edit), every mouse setting is passed so the launcher's stand.
    if let Some(p) = &input.path {
        set("PS3_INPUT_MAP", p.clone());
    }
    if matches!(s.input_mode.as_str(), "controller" | "keyboard") {
        set("PS3_INPUT_MODE", s.input_mode.clone());
    }
    let all = input.has_mouse_section;
    if !s.mouse {
        set("PS3_MOUSE", "0".into());
    } else {
        if all {
            set("PS3_MOUSE", "1".into());
        }
        if all || (s.mouse_sens - 1.0).abs() > 1e-3 {
            set("PS3_MOUSE_SENS", format!("{:.2}", s.mouse_sens.clamp(0.05, 10.0)));
        }
        if all || s.mouse_invert_y {
            set("PS3_MOUSE_INVERT_Y", if s.mouse_invert_y { "1" } else { "0" }.into());
        }
        if all || s.mouse_invert_x {
            set("PS3_MOUSE_INVERT_X", if s.mouse_invert_x { "1" } else { "0" }.into());
        }
        if all || s.mouse_deadzone != 30 {
            set("PS3_MOUSE_DEADZONE", s.mouse_deadzone.min(120).to_string());
        }
        if all || !s.mouse_capture {
            set("PS3_MOUSE_CAPTURE", if s.mouse_capture { "1" } else { "0" }.into());
        }
        if s.mouse_direct {
            set("RES_MOUSE_LOOK", "1".into());
        }
    }
    if !s.rumble {
        set("PS3_RUMBLE", "0".into());
    } else if s.rumble_strength != 100 {
        set("PS3_RUMBLE_STRENGTH", s.rumble_strength.min(200).to_string());
    }
    if !s.adaptive_triggers {
        set("PS3_PAD_ADAPTIVE", "0".into());
    }
    // Audio: only what differs from the default (full volume, the system's
    // output). The runtime mutes on any PS3_AUDIO_MUTE value, so it is
    // passed only when on.
    if s.audio_mute {
        set("PS3_AUDIO_MUTE", "1".into());
    } else if s.audio_volume < 100 {
        set("PS3_AUDIO_VOLUME", s.audio_volume.to_string());
    }
    let device = s.audio_device.trim();
    if !device.is_empty() && !device.contains(['\n', '\r']) {
        set("PS3_AUDIO_DEVICE", device.to_string());
    }
    // The optional game patches (the UI offers them only when the build has
    // them). 60 fps is the campaign's and the front end's together.
    if s.unlock_fps || s.menu_60fps {
        set("RES_UNLOCK_FPS", "1".into());
        set("RES_MENU_60FPS", "1".into());
    }
    if let Some(f) = fov_value(&s.fov) {
        set("RES_FOV", f);
    }
    if s.show_fps {
        set("PS3RECOMP_METAL_FPS", "1".into()); // exactly "1"
    }
    if !s.vertex_cache {
        set("PS3RECOMP_ENG_VCACHE", "0".into());
    } else if !s.vertex_cache_split {
        set("PS3RECOMP_ENG_VCACHE_SPLIT", "0".into());
    }
    if !s.bind_changed_only {
        set("PS3RECOMP_METAL_BIND_ALL", "1".into());
    }
    if let Some(d) = shader_cache_dir(s) {
        set("PS3_SHADER_CACHE", s_(&d));
    }
    if s.shader_async {
        set("PS3_SHADER_ASYNC", "1".into());
    }
    if s.frame_clock_us > 0 && s.frame_clock_us != 1000 {
        set("PS3_FRAMECLOCK_US", s.frame_clock_us.clamp(250, 16000).to_string());
    }
    if s.quiet_log {
        set("PS3_VERBOSE", "0".into());
    }
    env.extend(parse_env_lines(&s.extra_env));
    env
}

fn s_(p: &Path) -> String {
    s(p)
}

/// The program and arguments: [caffeinate -i] host [--size WxH] -- eboot elf;
/// on Windows the EBOOT build itself: eboot elf.
fn launch_argv(s: &Settings, b: &Builds) -> Result<Vec<String>, String> {
    let need = |v: &Option<String>, what: &str| v.clone().ok_or_else(|| format!("{what} is missing in {} (Advanced > Install / rebuild)", b.root));
    let eboot = need(&b.eboot, "the EBOOT build")?;
    let elf = need(&b.eboot_elf, "game/BCUS98107/EBOOT.elf")?;
    if !USES_HOST {
        return Ok(vec![eboot, elf]);
    }
    let host = need(&b.host, "the window host build/BCUS98107/Resistance")?;
    let mut argv = Vec::new();
    if s.keep_awake && Path::new("/usr/bin/caffeinate").is_file() {
        argv.extend(["/usr/bin/caffeinate".to_string(), "-i".to_string()]);
    }
    argv.push(host);
    if valid_window_size(&s.window_size) {
        argv.extend(["--size".to_string(), s.window_size.clone()]);
    }
    argv.extend(["--".to_string(), eboot, elf]);
    Ok(argv)
}

/// The equivalent command line, shown in Advanced > Environment.
#[tauri::command]
fn launch_preview(settings: Settings) -> Result<String, String> {
    let root = port_root(&settings).ok_or("no port folder yet (Advanced > Install / rebuild)")?;
    let b = resolve_builds(&root);
    let disc = find_disc(&settings).map(|d| s(&d)).unwrap_or_else(|| "<your mounted disc>".into());
    let play = play_dir(&settings);
    let env = launch_env(&settings, Path::new(&disc), &play, &input_env());
    let mut out = vec![format!("cd {} &&", shell_quote(&s(&play)))];
    out.extend(env.into_iter().map(|(k, v)| format!("{k}={}", shell_quote(&v))));
    out.extend(launch_argv(&settings, &b).unwrap_or_else(|e| vec![format!("# {e}")]).iter().map(|a| shell_quote(a)));
    out.push(format!("> {} 2>&1", shell_quote(&s(&play.join("last.log")))));
    Ok(out.join(" "))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayStarted {
    log: String,
    backup: Option<String>,
    backup_error: Option<String>,
    report: Option<String>,
    /// The disc was mounted from the image for this game.
    mounted: Option<String>,
    first_boot: bool,
    campaign: String,
}

/// Why the game cannot start. The window explains NoDisc and UpdateFiles
/// itself; a headless `--play` (recomplauncher.rs) turns each into an exit code.
#[derive(Debug)]
pub(crate) enum LaunchError {
    /// No disc set or mounted (and no image to attach).
    NoDisc,
    /// The disc in use is another game's.
    WrongDisc(String),
    /// The v1.7 update's files are in the play folder (see update_files).
    UpdateFiles,
    /// No port folder, or its builds are incomplete.
    NotBuilt(String),
    /// Online play: no valid online ID set.
    OnlineId,
    /// Online play: the game has downloaded its v1.7 update, and the v1.7
    /// menus are not built from it yet.
    OnlineBuild,
    Other(String),
}

impl LaunchError {
    /// The window's form: "no-disc", "update-files", "online-id" and
    /// "online-build" as codes, else the message.
    fn window(self) -> String {
        match self {
            LaunchError::NoDisc => "no-disc".into(),
            LaunchError::UpdateFiles => "update-files".into(),
            LaunchError::OnlineId => "online-id".into(),
            LaunchError::OnlineBuild => "online-build".into(),
            LaunchError::WrongDisc(m) | LaunchError::NotBuilt(m) | LaunchError::Other(m) => m,
        }
    }
}

impl From<String> for LaunchError {
    fn from(e: String) -> Self {
        LaunchError::Other(e)
    }
}

/// Everything Play does before the game starts, shared by the window's Play
/// and the headless `--play`: the checks, the disc (attached when needed), the
/// log, the saves' backup, and the game's command line, not started yet.
pub(crate) struct Launch {
    cmd: Command,
    argv: Vec<String>,
    env: Vec<(String, String)>,
    log_path: PathBuf,
    report_dir: Option<PathBuf>,
    backup: Option<String>,
    backup_error: Option<String>,
    mounted: Option<String>,
    first_boot: bool,
    builds: Builds,
    disc: PathBuf,
}

/// `diagnostic`: debug mode (report.rs). `keep_awake_wrapper`: `caffeinate -i`
/// in front of the game when the setting asks for it (the headless `--play`
/// keeps the Mac awake its own way, since it replaces itself with the game).
pub(crate) fn prepare_launch(s: &Settings, diagnostic: bool, keep_awake_wrapper: bool) -> Result<Launch, LaunchError> {
    let root = port_root(s).ok_or_else(|| LaunchError::NotBuilt("no port folder: pick it in Advanced > Install / rebuild".into()))?;
    let b = resolve_builds(&root);
    if !b.ok {
        return Err(LaunchError::NotBuilt(format!("missing in {}: {}", b.root, b.missing.join(", "))));
    }
    let play = play_dir(s);
    let stray = update_files(&play);
    if !stray.is_empty() {
        return Err(LaunchError::UpdateFiles); // the window explains and offers to move them aside
    }
    let mut mounted = None;
    let disc = match find_disc(s) {
        Some(d) => d,
        None if s.auto_mount && !s.disc_image.trim().is_empty() => {
            let d = attach_image(Path::new(s.disc_image.trim()))?;
            mounted = Some(self::s(&d));
            d
        }
        None => return Err(LaunchError::NoDisc),
    };
    if disc_title_id(&disc).as_deref() != Some(TITLE_ID) {
        return Err(LaunchError::WrongDisc(format!("{}: not a {TITLE_ID} disc", disc.display())));
    }
    for d in ["dev_hdd0", "dev_hdd1"] {
        fs::create_dir_all(play.join(d)).map_err(|e| format!("{}: {e}", play.display()))?;
    }
    let first = first_boot(&play);
    let report_dir = if diagnostic { Some(report::create(&reports_dir(), &saves::stamp(now_secs()))?) } else { None };
    let log_path = match &report_dir {
        Some(d) => d.join(report::LOG),
        None => play.join("last.log"),
    };
    let log = fs::File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
    let log2 = log.try_clone().map_err(|e| e.to_string())?;

    // the saves as they are now, before the game can touch them
    let (backup, backup_error) = match saves::backup_before_play(&save_locations(s), s.save_backups_keep, now_secs()) {
        Ok(b) => (b.map(|b| b.id), None),
        Err(e) => (None, Some(e)),
    };

    let argv = if keep_awake_wrapper { launch_argv(s, &b)? } else { launch_argv(&Settings { keep_awake: false, ..s.clone() }, &b)? };
    let mut env = launch_env(s, &disc, &play, &input_env());
    if let Some(d) = &report_dir {
        env.extend(report::env(d)); // after the extra variables: the report's paths win
    }
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..])
        .current_dir(&play)
        .env("PATH", tool_path())
        .env_remove("PS3RECOMP_METAL_HEADLESS")
        .env_remove("PS3RECOMP_HOST_SURFACES")
        .env_remove("PS3RECOMP_WINDOW_STATE")
        .env_remove("RES_GAME_VERSION")
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(log2);
    for (k, v) in &env {
        cmd.env(k, v);
    }
    Ok(Launch { cmd, argv, env, log_path, report_dir, backup, backup_error, mounted, first_boot: first, builds: b, disc })
}

/// Start the game. `diagnostic`: debug mode (report.rs), with its log and
/// dumps in a new report folder.
#[tauri::command]
fn play(app: AppHandle, state: State<'_, AppState>, diagnostic: Option<bool>) -> Result<PlayStarted, String> {
    require_supported_host()?;
    refuse_while_updating(&state)?;
    let mut slot = state.game.lock().unwrap();
    if slot.is_some() {
        return Err("the game is already running".into());
    }
    let s = load_settings_file();
    let Launch { mut cmd, argv, env, log_path, report_dir, backup, backup_error, mounted, first_boot: first, builds: b, disc } =
        prepare_launch(&s, diagnostic.unwrap_or(false), true).map_err(LaunchError::window)?;
    own_process_group(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("{}: {e}", argv[0]))?;
    *slot = Some(child.id());
    drop(slot);
    *state.last_log.lock().unwrap() = Some(log_path.clone());
    let started = now_secs();
    if let Some(d) = &report_dir {
        *state.report.lock().unwrap() = Some((d.clone(), true));
        session_event(d, "start", &format!("pid={}", child.id()));
        let (d2, b2, disc2, now) = (d.clone(), b.clone(), disc.clone(), started);
        let env2 = env.clone();
        let argv2 = argv.clone();
        thread::spawn(move || {
            let _ = fs::write(d2.join(report::SYSTEM), system_info(&b2, &disc2, &env2, &argv2, now));
        });
        spawn_ticker(app.clone(), d.clone());
    }
    let a = app.clone();
    let session = report_dir.clone();
    thread::spawn(move || {
        let status = child.wait();
        let code = status.as_ref().ok().and_then(|s| s.code()).unwrap_or(-1);
        let st = a.state::<AppState>();
        *st.game.lock().unwrap() = None;
        if let Some((_, live)) = st.report.lock().unwrap().as_mut() {
            *live = false;
        }
        let _ = a.emit("game-exit", code);
        let debug = session.is_some();
        if let Some(d) = session {
            session_exit(&d, status.ok(), started);
            let _ = a.emit("report-changed", ());
        }
        if code == 0 && !debug && *st.exit_with_game.lock().unwrap() {
            a.exit(0);
        }
    });
    let campaign = if s.campaign == "100" || !b.v17 { "01.00" } else { "v1.7" };
    Ok(PlayStarted {
        log: s_(&log_path),
        backup,
        backup_error,
        report: report_dir.map(|d| s_(&d)),
        mounted,
        first_boot: first,
        campaign: campaign.into(),
    })
}

// ---------------------------------------------------------------------------
// Online play (PS Rewired)
// ---------------------------------------------------------------------------

/// PS Rewired, the community revival whose servers run Resistance's online
/// play: its DNS answers the game's old server names with its servers, and
/// these three are the only addresses the game may reach online (the login,
/// the update download, the lobby and the games; PS3_NET_ALLOW). See the
/// ROADMAP, "Online multiplayer through PS Rewired".
const ONLINE_DNS: &str = "67.222.156.250";
const ONLINE_ALLOW: &str = "67.222.156.250,67.222.156.251,67.222.156.252";
/// The LAN address the game is told it has, instead of this computer's own.
const ONLINE_LOCAL_IP: &str = "192.168.1.100";

/// Online play's own folder: its dev_hdd0 (with the v1.7 update the game
/// downloads there, which the offline 01.00 menus cannot run with), dev_hdd1,
/// its profile and saves, overlay.ini and last.log. Nothing of the offline
/// game is touched by playing online.
fn online_dir(s: &Settings) -> PathBuf {
    play_dir(s).join("online")
}

/// PSN online-ID rules: 3-16 letters, digits, '-' or '_', starting with a letter.
fn valid_online_id(id: &str) -> bool {
    let b = id.as_bytes();
    (3..=16).contains(&b.len()) && b[0].is_ascii_alphabetic() && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'-' || *c == b'_')
}

/// The update's EBOOT.BIN, where the game puts it when it downloads its update.
fn online_update_eboot(s: &Settings) -> PathBuf {
    game_data_dir(&online_dir(s)).join("USRDIR/EBOOT.BIN")
}

/// What online play never takes from the settings or the extra variables:
/// the frame-rate patches (PS Rewired's rules forbid changing the frame rate
/// at any time), the 01.00 campaign switch, and the network and sign-in
/// variables, which online_env sets itself.
fn online_forbidden(k: &str) -> bool {
    matches!(k, "RES_UNLOCK_FPS" | "RES_MENU_60FPS" | "PS3_VBLANK_HZ" | "PS3_FLIP_MAX_HZ" | "RES_GAME_VERSION" | "PSNR_SERVER" | "PS3_MIC" | "PS3_MIC_TONE")
        || k.starts_with("PS3_NET_")
        || k.starts_with("PS3_NP_")
}

/// Play's environment for the online folder, without the frame-rate patches,
/// then the online settings: real sockets, signed in as the online ID with a
/// locally made ticket, PS Rewired's DNS, and the guards (only its addresses,
/// a generic LAN address, no UPnP requests to the router).
fn online_env(s: &Settings, disc: &Path, online: &Path, input: &InputEnv) -> Vec<(String, String)> {
    let mut os = s.clone();
    os.unlock_fps = false;
    os.menu_60fps = false;
    let mut env: Vec<(String, String)> = launch_env(&os, disc, online, input).into_iter().filter(|(k, _)| !online_forbidden(k)).collect();
    for (k, v) in [
        ("PS3_NET_ONLINE", "1"),
        ("PS3_NP_SIGNIN", "1"),
        ("PS3_NP_ONLINE_ID", s.online_id.trim()),
        ("PS3_NET_DNS", ONLINE_DNS),
        ("PS3_NET_ALLOW", ONLINE_ALLOW),
        ("PS3_NET_LOCAL_IP", ONLINE_LOCAL_IP),
        ("PS3_NET_NO_UPNP", "1"),
        ("PS3_NET_HOSTS", "localhost=127.0.0.1"),
    ] {
        env.push((k.to_string(), v.to_string()));
    }
    // the microphone only when the player turned voice chat on
    if s.voice_chat {
        env.push(("PS3_MIC".into(), "1".into()));
    }
    env
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineStarted {
    log: String,
    mounted: Option<String>,
    first_boot: bool,
    /// "v1.7": the v1.7 menus, straight online; "update": the disc's menus,
    /// so the game can download the update the servers require.
    menus: String,
}

/// The variables online play always takes out of the environment the game
/// would inherit (the launcher's own, or what started it: a terminal,
/// RecompLauncher), so that only online_env decides them: the frame-rate
/// patches and every other online_forbidden name, plus the window host's.
fn online_inherited_removals(inherited: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut v: Vec<String> = ["PS3RECOMP_METAL_HEADLESS", "PS3RECOMP_HOST_SURFACES", "PS3RECOMP_WINDOW_STATE", "RES_GAME_VERSION", "RES_UNLOCK_FPS", "RES_MENU_60FPS", "PS3_VBLANK_HZ", "PS3_FLIP_MAX_HZ", "PSNR_SERVER", "PS3_MIC", "PS3_MIC_TONE"]
        .iter()
        .map(|k| k.to_string())
        .collect();
    for k in inherited {
        if online_forbidden(&k) && !v.contains(&k) {
            v.push(k);
        }
    }
    v
}

/// Everything Play online does before the game starts, shared by the
/// window's Play online and the headless `--play-online` (recomplauncher.rs):
/// the checks, the disc (attached when needed), the online folder, its log,
/// and the game's command line with online_env, not started yet.
pub(crate) struct OnlineLaunch {
    cmd: Command,
    argv: Vec<String>,
    log_path: PathBuf,
    mounted: Option<String>,
    first_boot: bool,
    /// The v1.7 menus (else the disc's, for the update's download).
    v17: bool,
}

/// The v1.7 menus in the online folder with online_env. Before they are
/// built, the disc's menus, so that Online Game downloads the update (the
/// EBOOT.BIN the v1.7 menus are built from). `keep_awake_wrapper` as in
/// prepare_launch. Never a frame-rate setting, whatever the settings, the
/// extra variables or the inherited environment say.
pub(crate) fn prepare_online_launch(s: &Settings, input: &InputEnv, keep_awake_wrapper: bool) -> Result<OnlineLaunch, LaunchError> {
    if !valid_online_id(s.online_id.trim()) {
        return Err(LaunchError::OnlineId);
    }
    let root = port_root(s).ok_or_else(|| LaunchError::NotBuilt("no port folder: pick it in Advanced > Install / rebuild".into()))?;
    let b = resolve_builds(&root);
    if !b.ok {
        return Err(LaunchError::NotBuilt(format!("missing in {}: {}", b.root, b.missing.join(", "))));
    }
    let v17 = b.eboot_v17.is_some();
    if !v17 && online_update_eboot(s).is_file() {
        return Err(LaunchError::OnlineBuild);
    }
    let mut mounted = None;
    let disc = match find_disc(s) {
        Some(d) => d,
        None if s.auto_mount && !s.disc_image.trim().is_empty() => {
            let d = attach_image(Path::new(s.disc_image.trim()))?;
            mounted = Some(self::s(&d));
            d
        }
        None => return Err(LaunchError::NoDisc),
    };
    if disc_title_id(&disc).as_deref() != Some(TITLE_ID) {
        return Err(LaunchError::WrongDisc(format!("{}: not a {TITLE_ID} disc", disc.display())));
    }
    let online = online_dir(s);
    for d in ["dev_hdd0", "dev_hdd1"] {
        fs::create_dir_all(online.join(d)).map_err(|e| format!("{}: {e}", online.display()))?;
    }
    // The first boot's install cache, from the offline folder: saves the
    // "initial setup" a second time (the cache is the disc's data, no save).
    let cache = game_data_dir(&online).join("USRDIR/cache");
    let offline_cache = game_data_dir(&play_dir(s)).join("USRDIR/cache");
    if !cache.is_dir() && offline_cache.is_dir() {
        let _ = saves::copy_tree(&offline_cache, &cache, &[]);
    }
    let first = first_boot(&online);
    let log_path = online.join("last.log");
    let log = fs::File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
    let log2 = log.try_clone().map_err(|e| e.to_string())?;

    let mut menus = b.clone();
    if v17 {
        menus.eboot = b.eboot_v17.clone();
        menus.eboot_elf = b.eboot_v17_elf.clone();
    }
    let argv = if keep_awake_wrapper { launch_argv(s, &menus)? } else { launch_argv(&Settings { keep_awake: false, ..s.clone() }, &menus)? };
    let env = online_env(s, &disc, &online, input);
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]).current_dir(&online).env("PATH", tool_path()).stdin(Stdio::null()).stdout(log).stderr(log2);
    for k in online_inherited_removals(std::env::vars_os().map(|(k, _)| k.to_string_lossy().into_owned())) {
        cmd.env_remove(k);
    }
    for (k, v) in &env {
        cmd.env(k, v);
    }
    Ok(OnlineLaunch { cmd, argv, log_path, mounted, first_boot: first, v17 })
}

/// "Play online": prepare_online_launch, started by the window. When the
/// game exits with the update downloaded, "online-update-ready" asks for
/// build_online_menus. Errors the window explains itself: "online-id" (none
/// set), "online-build" (the update is there, the menus not built yet),
/// "no-disc".
#[tauri::command]
fn play_online(app: AppHandle, state: State<'_, AppState>) -> Result<OnlineStarted, String> {
    require_supported_host()?;
    refuse_while_updating(&state)?;
    let mut slot = state.game.lock().unwrap();
    if slot.is_some() {
        return Err("the game is already running".into());
    }
    if state.rebuild.lock().unwrap().is_some() {
        return Err("a rebuild is running".into());
    }
    let s = load_settings_file();
    let OnlineLaunch { mut cmd, argv, log_path, mounted, first_boot: first, v17 } = prepare_online_launch(&s, &input_env(), true).map_err(LaunchError::window)?;
    own_process_group(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("{}: {e}", argv[0]))?;
    *slot = Some(child.id());
    drop(slot);
    *state.last_log.lock().unwrap() = Some(log_path.clone());
    let a = app.clone();
    let update_eboot = online_update_eboot(&s);
    thread::spawn(move || {
        let status = child.wait();
        let code = status.as_ref().ok().and_then(|s| s.code()).unwrap_or(-1);
        *a.state::<AppState>().game.lock().unwrap() = None;
        let _ = a.emit("game-exit", code);
        if !v17 && update_eboot.is_file() {
            let _ = a.emit("online-update-ready", ());
        }
    });
    Ok(OnlineStarted {
        log: s_(&log_path),
        mounted,
        first_boot: first,
        menus: if v17 { "v1.7" } else { "update" }.into(),
    })
}

/// SHA-256 of the v1.7 EBOOT.BIN PS Rewired's update server hands out, the
/// one tools/v17/regen_eboot_v17.sh was tested with (which checks its
/// decrypted ELF too). It comes from a community server and becomes native
/// code, so build_online_menus refuses any other.
const ONLINE_EBOOT_SHA: &str = "5edb729f7d8d251ff1f2f9a177a8c8af941d56c54c3df773cd5c6b37192d8ef7";

fn file_sha256(p: &Path) -> Option<String> {
    let o = Command::new("/usr/bin/shasum").args(["-a", "256"]).arg(p).output().ok()?;
    if !o.status.success() {
        return None;
    }
    String::from_utf8(o.stdout).ok()?.split_whitespace().next().map(str::to_string)
}

/// The script build_online_menus runs in the port folder: the toolkit and
/// the venv as the rebuild sets them up, then tools/rebuild.sh eboot17.
fn online_menus_script(deps: bool) -> String {
    rebuild_script(&["toolkit".to_string()], deps).replace("echo '==> done'\n", "PY=$PWD/.venv/bin/python exec tools/rebuild.sh eboot17\n")
}

/// Build the v1.7 menus from the update the game downloaded into the online
/// folder (tools/v17/regen_eboot_v17.sh), streamed like a rebuild.
#[tauri::command]
fn build_online_menus(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    require_supported_host()?;
    refuse_while_updating(&state)?;
    if cfg!(windows) {
        return Err("building the v1.7 menus needs macOS".into());
    }
    if state.game.lock().unwrap().is_some() {
        return Err("the game is running: close it first".into());
    }
    let mut slot = state.rebuild.lock().unwrap();
    if slot.is_some() {
        return Err("a rebuild is already running".into());
    }
    let (mut cmd, first) = online_menus_job(&load_settings_file())?;
    // the install lock while it builds, as RecompLauncher's setup and its
    // --play-online build the same menus (recomplauncher.rs)
    let lock = match install_lock() {
        Ok(Some(l)) => Some(l),
        Ok(None) => return Err("an install or a build of the online menus is already running".into()),
        Err(_) => None,
    };
    own_process_group(&mut cmd);
    let child = cmd.spawn().map_err(|e| format!("zsh: {e}"))?;
    *slot = Some(child.id());
    drop(slot);
    let _ = app.emit("rebuild-line", first);
    stream_child_logged(&app, child, "rebuild", |s| &s.rebuild, None, lock);
    Ok(())
}

/// The v1.7 menus' build from the update in the online folder, checked and
/// not started yet (its output piped), with the line that announces it.
/// Shared by build_online_menus and the headless `--prepare` and
/// `--play-online` (recomplauncher.rs).
pub(crate) fn online_menus_job(st: &Settings) -> Result<(Command, String), String> {
    let root = port_root(st).ok_or("no port folder (pick the resistance-recomp checkout)")?;
    if !root.join("tools/v17/regen_eboot_v17.sh").is_file() {
        return Err(format!("{}: no tools/v17/regen_eboot_v17.sh (Get the new sources first)", root.display()));
    }
    let eboot = online_update_eboot(st);
    if !eboot.is_file() {
        return Err("no update downloaded yet: Play online, then Multiplayer > Online Game, and let the game download its update".into());
    }
    if file_sha256(&eboot).as_deref() != Some(ONLINE_EBOOT_SHA) {
        return Err(format!("{}: not the tested v1.7 EBOOT.BIN, so it is not built (it comes from a server and would become native code)", eboot.display()));
    }
    let deps = !deps_ready(&deps_dir()) && !brew_complete() && root.join("tools/bootstrap_deps.sh").is_file();
    let mut cmd = Command::new("/usr/bin/nice");
    cmd.args(["-n", "10", "/bin/zsh", "-c", &online_menus_script(deps)])
        .current_dir(&root)
        .env("PATH", tool_path())
        .env("RES_DEPS", deps_dir())
        .env("RES_EBOOT17", &eboot)
        .env("TERM", "dumb")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    Ok((cmd, format!("$ cd {} && tools/rebuild.sh eboot17   # the v1.7 menus, for online play", root.display())))
}

/// The headless online-menus build's output (the window's goes to its
/// rebuild console).
fn online_menus_log_path() -> PathBuf {
    data_root().join("online-menus.log")
}

/// Stop: SIGTERM to what Play started (the window host closes the game, then
/// itself). The host can miss its game's exit and linger with the window
/// open after a SIGTERM; if that group is still there 8 s later it gets
/// SIGKILL. Only this launcher's own process group, never by name.
#[tauri::command]
fn stop_game(app: AppHandle, state: State<'_, AppState>) {
    if let Some(pid) = *state.game.lock().unwrap() {
        if let Some((d, true)) = state.report.lock().unwrap().clone() {
            session_event(&d, "stop", "");
        }
        kill_tree(pid, false);
        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_secs(8));
            if *app.state::<AppState>().game.lock().unwrap() == Some(pid) {
                kill_tree(pid, true);
            }
        });
    }
}

/// Move the update's patch_*.bin files out of the play folder's USRDIR/data
/// into `<play>/update-files-aside/<stamp>/` (kept, not deleted).
#[tauri::command]
fn move_update_files(state: State<'_, AppState>) -> Result<String, String> {
    if state.game.lock().unwrap().is_some() {
        return Err("the game is running".into());
    }
    let play = play_dir(&load_settings_file());
    let files = update_files(&play);
    if files.is_empty() {
        return Ok(String::new());
    }
    let dest = play.join("update-files-aside").join(saves::stamp(now_secs()));
    fs::create_dir_all(&dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    for f in &files {
        let to = dest.join(f.file_name().unwrap());
        fs::rename(f, &to).map_err(|e| format!("{}: {e}", f.display()))?;
    }
    Ok(s(&dest))
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    builds: Option<Builds>,
    disc: DiscInfo,
    icon: Option<String>,
    background: Option<String>,
    last_save: Option<SaveInfo>,
    game_running: bool,
    /// A ResistanceRecomp this launcher did not start (a terminal, jogar.sh).
    other_game_running: bool,
    rebuild_running: bool,
    play_dir: String,
    data_root: String,
    first_boot: bool,
    update_files: Vec<String>,
    /// Online play: its folder, whether the game has downloaded the v1.7
    /// update there, and whether the v1.7 menus are built.
    online_dir: String,
    online_update: bool,
    online_menus: bool,
    host_supported: bool,
    /// The port's build for this system is untested (Windows).
    experimental: bool,
    os: String,
    app_bundle: Option<String>,
    finder_icon: bool,
    version: String,
    /// FEATURE_MARKERS found in the builds.
    features: Vec<String>,
}

#[tauri::command]
async fn get_status(app: AppHandle, state: State<'_, AppState>) -> Result<Status, String> {
    let s = load_settings_file();
    let game_running = state.game.lock().unwrap().is_some();
    let rebuild_running = state.rebuild.lock().unwrap().is_some();
    let app2 = app.clone();
    let st = tauri::async_runtime::spawn_blocking(move || {
        let builds = port_root(&s).map(|r| resolve_builds(&r));
        let features = builds.as_ref().map(|b| build_features(b, &app2.state::<AppState>().features)).unwrap_or_default();
        let (icon, pic1) = disc_art(&s);
        let play = play_dir(&s);
        sync_app_icon(&app2);
        let bundle = app_icon::app_bundle().filter(|_| cfg!(target_os = "macos"));
        let online_menus = builds.as_ref().is_some_and(|b| b.eboot_v17.is_some());
        Status {
            builds,
            disc: disc_info(&s),
            icon: icon.and_then(|p| data_url(&p, "image/png")),
            background: pic1.and_then(|p| data_url(&p, "image/png")),
            last_save: last_save(&s),
            game_running,
            other_game_running: !game_running && (process_running(GAME_EXE) || process_running(LEGACY_EXE)),
            rebuild_running,
            first_boot: first_boot(&play),
            update_files: update_files(&play).iter().map(|p| self::s(p)).collect(),
            online_dir: self::s(&online_dir(&s)),
            online_update: online_update_eboot(&s).is_file(),
            online_menus,
            play_dir: self::s(&play),
            data_root: self::s(&data_root()),
            host_supported: host_supported(),
            experimental: cfg!(windows),
            os: std::env::consts::OS.into(),
            finder_icon: bundle.as_deref().is_some_and(app_icon::has_finder_icon),
            app_bundle: bundle.map(|b| self::s(&b)),
            version: env!("CARGO_PKG_VERSION").into(),
            features,
        }
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(st)
}

// ---------------------------------------------------------------------------
// What the build supports
// ---------------------------------------------------------------------------

/// Environment variables whose names in a build's executable say that it has
/// a feature the settings drive. A build made before the feature landed lacks
/// the name; the window then labels (or, for the optional game patches,
/// hides) the setting rather than pass a variable nothing reads.
const FEATURE_MARKERS: &[&str] = &[
    "PS3_INPUT_MAP",     // keyboard/mouse bindings, mouse camera, rumble
    "PS3_LUT_STRENGTH",  // SMAA and the present pass's filters
    "PS3_RES_SCALE",     // internal resolution
    "PS3_MODS_DIR",      // texture packs
    "PS3_WINDOW_MODE",   // the window host's modes, display, vsync
    "PS3_UPSCALE",       // MetalFX in the window host
    "PS3_AUDIO_VOLUME",  // volume, mute and output device
    "RES_UNLOCK_FPS",    // port/patches.cpp
    "RES_MENU_60FPS",
    "RES_FOV",
    "PS3_PAD_ADAPTIVE",  // DualSense adaptive triggers
    "RES_MOUSE_LOOK",    // direct mouse look (port/mouselook.cpp)
];

/// The markers found in `data` (a NUL-terminated C string in the binary).
fn markers_in(data: &[u8], found: &mut std::collections::BTreeSet<String>) {
    for m in FEATURE_MARKERS {
        if found.contains(*m) {
            continue;
        }
        let mut pat = m.as_bytes().to_vec();
        pat.push(0);
        if memchr::memmem::find(data, &pat).is_some() {
            found.insert((*m).to_string());
        }
    }
}

type FeatureCache = BTreeMap<String, (u64, u64, Vec<String>)>;

/// The markers in the executables of `b` (host, EBOOT, campaign builds),
/// cached by path, size and mtime: the builds are ~90 MB each.
fn build_features(b: &Builds, cache: &Mutex<FeatureCache>) -> Vec<String> {
    let mut all = std::collections::BTreeSet::new();
    for p in [&b.host, &b.eboot, &b.game, &b.game_v17].into_iter().flatten() {
        let meta = fs::metadata(p).ok();
        let key = (meta.as_ref().map_or(0, |m| m.len()), meta.as_ref().map_or(0, |m| mtime_of(m)));
        let hit = cache.lock().unwrap().get(p).filter(|c| (c.0, c.1) == key).map(|c| c.2.clone());
        let found = match hit {
            Some(v) => v,
            None => {
                let mut set = std::collections::BTreeSet::new();
                if let Ok(data) = fs::read(p) {
                    markers_in(&data, &mut set);
                }
                let v: Vec<String> = set.into_iter().collect();
                cache.lock().unwrap().insert(p.clone(), (key.0, key.1, v.clone()));
                v
            }
        };
        all.extend(found);
    }
    all.into_iter().collect()
}

fn mtime_of(m: &fs::Metadata) -> u64 {
    m.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Controls, screens, texture packs
// ---------------------------------------------------------------------------

/// The controls file for the Controls page: what it binds (the page fills in
/// the defaults for the rest), and where it is.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMapView {
    path: String,
    exists: bool,
    bindings: Bindings,
}

#[tauri::command]
fn get_input_map() -> InputMapView {
    let p = input_map_path();
    let text = fs::read_to_string(&p);
    InputMapView { path: s(&p), exists: text.is_ok(), bindings: text.map(|t| input_map::parse(&t).bindings).unwrap_or_default() }
}

/// Write the bindings into `path`; the file's other sections ([mouse],
/// anything newer) stay as they were.
fn write_input_map(path: &Path, bindings: Bindings) -> Result<(), String> {
    input_map::validate(&bindings)?;
    let mut file = fs::read_to_string(path).map(|t| input_map::parse(&t)).unwrap_or_default();
    file.bindings = bindings;
    if let Some(d) = path.parent() {
        fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, input_map::render(&file)).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_input_map(bindings: Bindings) -> Result<String, String> {
    let p = input_map_path();
    write_input_map(&p, bindings)?;
    Ok(s(&p))
}

/// "Restore defaults": the file goes (its other sections with it, kept in
/// input.toml.bak), so Play passes no PS3_INPUT_MAP again.
#[tauri::command]
fn reset_input_map() -> Result<(), String> {
    let p = input_map_path();
    if p.is_file() {
        fs::rename(&p, p.with_extension("toml.bak")).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The names in the JSON array the screen probe prints.
fn parse_screens(json: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(json.trim()).unwrap_or_default()
}

/// The screens in the system's order (NSScreen.screens: the first has the
/// menu bar), by the names the window host's PS3_DISPLAY matches.
#[tauri::command]
async fn list_screens() -> Vec<String> {
    if !cfg!(target_os = "macos") {
        return Vec::new();
    }
    tauri::async_runtime::spawn_blocking(|| {
        command_stdout(Command::new("/usr/bin/osascript").args([
            "-l",
            "JavaScript",
            "-e",
            "ObjC.import('AppKit'); JSON.stringify($.NSScreen.screens.js.map(s => s.localizedName.js))",
        ]))
        .map(|s| parse_screens(&s))
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModsView {
    dir: String,
    dump_dir: String,
    packs: Vec<mods::ModPack>,
}

/// The texture packs in <data>/mods (the folder is created, so "Open the
/// mods folder" always has somewhere to go).
#[tauri::command]
fn list_mods() -> ModsView {
    let dir = mods_dir();
    ModsView { packs: mods::list(&dir), dir: s(&dir), dump_dir: s(&mods_dump_dir()) }
}

#[tauri::command]
fn get_settings() -> Settings {
    load_settings_file()
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    write_settings(&settings)?;
    sync_app_icon(&app);
    Ok(())
}

#[tauri::command]
async fn find_port_dirs() -> Vec<Builds> {
    tauri::async_runtime::spawn_blocking(find_port_dirs_now).await.unwrap_or_default()
}

#[tauri::command]
fn builds_at(path: String) -> Builds {
    resolve_builds(Path::new(path.trim()))
}

#[tauri::command]
async fn disc_status() -> DiscInfo {
    tauri::async_runtime::spawn_blocking(|| disc_info(&load_settings_file())).await.unwrap_or_default()
}

/// Attach the image and remember it (and its mount point) as the disc.
#[tauri::command]
async fn mount_image(path: String) -> Result<DiscInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let root = attach_image(Path::new(path.trim()))?;
        let mut st = load_settings_file();
        st.disc_image = path.trim().to_string();
        st.disc_path = s(&root);
        write_settings(&st)?;
        Ok(disc_info(&st))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Eject the disc when it is a mounted image (refused while the game runs).
#[tauri::command]
async fn eject_disc(state: State<'_, AppState>) -> Result<DiscInfo, String> {
    if state.game.lock().unwrap().is_some() || process_running(GAME_EXE) {
        return Err("the game is running: close it first".into());
    }
    tauri::async_runtime::spawn_blocking(|| {
        let st = load_settings_file();
        let root = find_disc(&st).ok_or("no disc mounted")?;
        if cfg!(windows) {
            let letter = drive_letter(&root).ok_or_else(|| format!("{}: not a mounted disc image", root.display()))?;
            detach_windows(letter)?;
            return Ok(disc_info(&st));
        }
        if !root.starts_with("/Volumes") || image_of(&root).is_none() {
            return Err(format!("{}: not a mounted disc image", root.display()));
        }
        let out = Command::new("/usr/bin/hdiutil").arg("detach").arg(&root).stdin(Stdio::null()).output().map_err(|e| format!("hdiutil: {e}"))?;
        if !out.status.success() {
            return Err(format!("hdiutil detach: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(disc_info(&st))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// The Dock and Finder icons from the player's disc (app_icon.rs)
// ---------------------------------------------------------------------------

fn app_icon_cache() -> PathBuf {
    data_root().join(app_icon::CACHE_NAME)
}

fn composed_icon(s: &Settings) -> Option<PathBuf> {
    if !s.game_icon {
        return None;
    }
    app_icon::refresh(&app_icon_cache(), disc_art(s).0.as_deref())
}

fn sync_app_icon(app: &AppHandle) {
    if !cfg!(target_os = "macos") {
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        let icon = composed_icon(&load_settings_file());
        let key = app_icon::key(icon.as_deref());
        let state = app.state::<AppState>();
        let mut cur = state.dock_icon.lock().unwrap();
        if *cur != key {
            *cur = key;
            app_icon::apply_dock_icon(&app, icon);
        }
    });
}

#[tauri::command]
async fn finder_icon(app: AppHandle, set: bool) -> Result<bool, String> {
    if !cfg!(target_os = "macos") {
        return Err("the Finder icon is macOS only".into());
    }
    let bundle = app_icon::app_bundle().ok_or("the launcher is not running from an .app (development build)")?;
    let icon = if set {
        let st = load_settings_file();
        Some(app_icon::refresh(&app_icon_cache(), disc_art(&st).0.as_deref()).ok_or("no ICON0.PNG from your disc yet")?)
    } else {
        None
    };
    let (tx, rx) = std::sync::mpsc::channel();
    let b = bundle.clone();
    app.run_on_main_thread(move || {
        let _ = tx.send(app_icon::set_finder_icon(&b, icon.as_deref()));
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())??;
    Ok(app_icon::has_finder_icon(&bundle))
}

#[tauri::command]
fn take_play_request(state: State<'_, AppState>) -> bool {
    std::mem::take(&mut *state.play_request.lock().unwrap())
}

/// `--play-online` (without RecompLauncher): the window starts online play
/// once, as the Play online button.
#[tauri::command]
fn take_online_request(state: State<'_, AppState>) -> bool {
    std::mem::take(&mut *state.online_request.lock().unwrap())
}

#[tauri::command]
fn take_start_view(state: State<'_, AppState>) -> Option<String> {
    state.start_view.lock().unwrap().take()
}

/// Home > Status > "Play shortcut on the Desktop" (play_shortcut.rs).
#[tauri::command]
async fn create_play_shortcut(app: AppHandle) -> Result<String, String> {
    if !cfg!(target_os = "macos") {
        return Err("the Play shortcut is macOS only".into());
    }
    let bundle = app_icon::app_bundle().ok_or("the launcher is not running from an .app (development build)")?;
    let icns = play_shortcut::bundle_icns(&bundle);
    let path = play_shortcut::write(&home().join("Desktop"), &app.config().identifier, icns.as_deref())?;
    if let Some(icon) = composed_icon(&load_settings_file()) {
        let (tx, rx) = std::sync::mpsc::channel();
        let p = path.clone();
        app.run_on_main_thread(move || {
            let _ = tx.send(app_icon::set_finder_icon(&p, Some(&icon)));
        })
        .map_err(|e| e.to_string())?;
        if let Ok(Err(e)) = rx.recv() {
            eprintln!("[launcher] Play shortcut icon: {e}");
        }
    }
    Ok(s(&path))
}

// ---------------------------------------------------------------------------
// Add to Steam (steam.rs, steam_art.rs)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamStatus {
    available: bool,
    installed: bool,
    running: bool,
    users: usize,
    added: usize,
}

fn steam_dir() -> PathBuf {
    data_root().join("steam")
}

fn steam_target() -> steam::Target {
    steam::Target::new(&play_shortcut::executable(&steam_dir().join(play_shortcut::NAME)))
}

fn steam_available() -> bool {
    cfg!(target_os = "macos") && app_icon::app_bundle().is_some()
}

fn steam_art(st: &Settings) -> Option<steam::Art> {
    let (icon0, pic1) = disc_art(st);
    let icon0 = app_icon::decode_png(&fs::read(icon0?).ok()?).ok()?;
    let pic1 = pic1.and_then(|p| fs::read(p).ok()).and_then(|b| app_icon::decode_png(&b).ok());
    let imgs = steam_art::draw(&icon0, pic1.as_ref());
    let enc = |i: &app_icon::Rgba| app_icon::encode_png(i).ok();
    Some([enc(&imgs.portrait)?, enc(&imgs.landscape)?, enc(&imgs.hero)?, enc(&imgs.icon)?])
}

#[tauri::command]
async fn steam_status() -> SteamStatus {
    tauri::async_runtime::spawn_blocking(|| {
        let users = steam::users(&steam::root(&home()));
        let t = steam_target();
        SteamStatus {
            available: steam_available(),
            installed: !users.is_empty(),
            running: steam::running(),
            users: users.len(),
            added: users.iter().filter(|u| steam::user_has(u, &t)).count(),
        }
    })
    .await
    .unwrap_or(SteamStatus { available: false, installed: false, running: false, users: 0, added: 0 })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamAdded {
    users: usize,
    art: bool,
}

#[tauri::command]
async fn steam_add(app: AppHandle) -> Result<SteamAdded, String> {
    if !steam_available() {
        return Err("Add to Steam needs the packaged app on macOS".into());
    }
    let id = app.config().identifier.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let users = steam::users(&steam::root(&home()));
        if users.is_empty() {
            return Err("Steam is not installed here, or nobody has signed in to it yet".to_string());
        }
        if steam::running() {
            return Err("steam-running".to_string());
        }
        let icns = app_icon::app_bundle().and_then(|b| play_shortcut::bundle_icns(&b));
        fs::create_dir_all(steam_dir()).map_err(|e| e.to_string())?;
        play_shortcut::write_kind(&steam_dir(), &id, icns.as_deref(), play_shortcut::Kind::Steam)?;
        let art = steam_art(&load_settings_file());
        let t = steam_target();
        for u in &users {
            steam::add(u, &t, art.as_ref())?;
        }
        Ok(SteamAdded { users: users.len(), art: art.is_some() })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn steam_remove(app: AppHandle) -> Result<usize, String> {
    let id = app.config().identifier.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if steam::running() {
            return Err("steam-running".to_string());
        }
        let t = steam_target();
        let mut n = 0;
        for u in &steam::users(&steam::root(&home())) {
            n += steam::remove(u, &t)? as usize;
        }
        let bundle = steam_dir().join(play_shortcut::NAME);
        if play_shortcut::is_ours(&bundle, &id, play_shortcut::Kind::Steam) {
            fs::remove_dir_all(&bundle).map_err(|e| format!("{}: {e}", bundle.display()))?;
        }
        Ok(n)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn steam_quit() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(steam::quit).await.map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Requirements and the rebuild
// ---------------------------------------------------------------------------

/// One line of Advanced > Install / rebuild > "What this computer needs".
/// `level`: "ok", "warn", "fail", "info" (not a problem, only to know) or
/// "optional" (absent, and not needed). The window shows `key` (an i18n key
/// under `prereq.msg.`, with `vars`) and falls back to the English
/// `message`; `path` is shown on a line of its own, shortened, with the whole
/// path as a tooltip; `optional` marks a tool that is only a fallback.
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CheckItem {
    level: String,
    name: String,
    message: String,
    key: String,
    vars: BTreeMap<String, String>,
    path: Option<String>,
    optional: bool,
}

impl CheckItem {
    fn new(level: &str, name: &str, key: &str, message: impl Into<String>) -> Self {
        CheckItem { level: level.into(), name: name.into(), key: key.into(), message: message.into(), ..Default::default() }
    }
    /// A check whose detail is only a path.
    fn at(level: &str, name: &str, path: &Path) -> Self {
        CheckItem::new(level, name, "", s(path)).path(path)
    }
    fn var(mut self, k: &str, v: impl ToString) -> Self {
        self.vars.insert(k.into(), v.to_string());
        self
    }
    fn path(mut self, p: &Path) -> Self {
        self.path = Some(s(p));
        self
    }
    fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Prereqs {
    supported: bool,
    clt: bool,
    checks: Vec<CheckItem>,
    ok: bool,
    /// tools/bootstrap_deps.sh's folder is complete, or Homebrew has everything.
    deps: bool,
    deps_dir: String,
    /// The port's sources (a folder with tools/rebuild.sh), when there are any;
    /// "Install the game" downloads them otherwise.
    sources: Option<String>,
    /// The game's executables are decrypted already (the disc is then only
    /// needed to play).
    elfs: bool,
    disc: bool,
    builds_ok: bool,
    free_gb: Option<u64>,
    need_gb: u64,
}

/// What the rebuild and the game need, by Homebrew formula: (formula, what
/// shows it is installed). Only the fallback when tools/bootstrap_deps.sh's
/// folder is not there.
const BREW_NEEDS: &[(&str, &[&str])] = &[
    ("cmake", &["bin/cmake"]),
    ("ninja", &["bin/ninja"]),
    ("sdl2", &["lib/libSDL2.dylib"]),
    ("glslang", &["bin/glslangValidator", "bin/glslang"]),
    ("spirv-cross", &["bin/spirv-cross"]),
    ("spirv-tools", &["bin/spirv-opt"]),
    ("ffmpeg", &["lib/libavcodec.dylib"]),
    ("python@3.12", &["bin/python3.12"]),
];

fn find_rpcs3() -> Option<PathBuf> {
    let places = if cfg!(windows) {
        let pf = std::env::var_os("ProgramFiles").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("C:\\Program Files"));
        vec![home().join("rpcs3/rpcs3.exe"), PathBuf::from("C:\\rpcs3\\rpcs3.exe"), pf.join("RPCS3/rpcs3.exe")]
    } else {
        vec![home().join("Applications/RPCS3.app/Contents/MacOS/rpcs3"), PathBuf::from("/Applications/RPCS3.app/Contents/MacOS/rpcs3")]
    };
    places.into_iter().find(|p| p.is_file()).or_else(|| find_in_path("rpcs3"))
}

fn free_gb(p: &Path) -> Option<u64> {
    if cfg!(windows) {
        return None;
    }
    let out = command_stdout(Command::new("/bin/df").arg("-k").arg(p))?;
    let kb: u64 = out.lines().nth(1)?.split_whitespace().nth(3)?.parse().ok()?;
    Some(kb / (1024 * 1024))
}

/// Homebrew has every formula the build needs.
fn brew_complete() -> bool {
    let prefix = brew_prefix();
    find_brew().is_some() && BREW_NEEDS.iter().all(|(_, marks)| marks.iter().any(|m| prefix.join(m).exists()))
}

/// The disk space a full install still needs here: none once the builds are there.
fn need_gb(b: Option<&Builds>) -> u64 {
    if b.is_some_and(|b| b.ok) {
        0
    } else {
        INSTALL_NEED_GB
    }
}

#[tauri::command]
async fn check_prereqs() -> Prereqs {
    tauri::async_runtime::spawn_blocking(|| {
        let st = load_settings_file();
        let dd = deps_dir();
        let mut p = Prereqs { supported: host_supported(), clt: clt_installed(), deps_dir: s(&dd), ..Default::default() };
        let mut add = |c: CheckItem| p.checks.push(c);
        if !p.supported {
            return p;
        }
        let mut deps = true;
        if cfg!(windows) {
            // the rebuild on Windows is untested: what tools/rebuild.sh calls,
            // found on PATH (a POSIX shell with zsh, e.g. MSYS2)
            for tool in ["git", "zsh", "cmake", "ninja", "python"] {
                match find_in_path(tool) {
                    Some(f) => add(CheckItem::at("ok", tool, &f)),
                    None => add(CheckItem::new("warn", tool, "toolMissing", "not found on PATH (needed only to rebuild)")),
                }
            }
        } else {
            add(if p.clt {
                CheckItem::new("ok", "xcode-clt", "cltOk", "installed")
            } else {
                CheckItem::new("fail", "xcode-clt", "cltMissing", "not installed (Apple's installer: the compiler and git)")
            });
            // the build tools and libraries: tools/bootstrap_deps.sh's, else Homebrew's
            if deps_ready(&dd) {
                add(CheckItem::new("ok", "deps", "depsOwn", "installed by tools/bootstrap_deps.sh").path(&dd));
            } else if brew_complete() {
                add(CheckItem::new("ok", "deps", "depsBrew", "Homebrew's").path(&brew_prefix()));
            } else {
                deps = false;
                add(CheckItem::new("warn", "deps", "depsLater", "not installed yet: the install downloads and builds them (a few minutes)"));
            }
        }
        p.deps = deps;
        let root = port_root(&st).filter(|r| r.join("tools/rebuild.sh").is_file() || resolve_builds(r).ok);
        let b = root.as_deref().map(resolve_builds);
        let elfs = b.as_ref().is_some_and(|b| b.eboot_elf.is_some() && b.game_elf.is_some());
        p.elfs = elfs;
        // The rebuild decrypts with tools/decrypt_self.py (keys downloaded once
        // from RPCS3's public source by tools/fetch_keys.py); RPCS3 is only its
        // fallback, so a missing RPCS3 is not a problem.
        add(match find_rpcs3() {
            Some(r) => CheckItem::at("ok", "rpcs3", &r).optional(),
            None if elfs => CheckItem::new("optional", "rpcs3", "rpcs3NotNeededElf", "not needed: the decrypted ELFs are there").optional(),
            None => CheckItem::new("optional", "rpcs3", "rpcs3NotNeededOwn", "not needed: the rebuild decrypts EBOOT.BIN and game.self itself (internet needed once)").optional(),
        });
        let disc = find_disc(&st);
        p.disc = disc.is_some();
        add(match disc {
            Some(d) => CheckItem::at("ok", "disc", &d),
            None if elfs => CheckItem::new("warn", "disc", "discForPlay", "not mounted (needed to play)"),
            None => CheckItem::new("fail", "disc", "discMissing", "not mounted (needed to install and to play)"),
        });
        match &b {
            Some(b) if b.rebuild_sh.is_some() => {
                let root = Path::new(&b.root);
                p.sources = Some(b.root.clone());
                let tk = root.join("ps3recomp");
                add(if tk.join(".git").exists() { CheckItem::at("ok", "toolkit", &tk) } else { CheckItem::new("warn", "toolkit", "toolkitLater", "not set up yet (the toolkit step does it)") });
                let venv = root.join(".venv");
                add(if venv.join("bin/python").is_file() { CheckItem::at("ok", "venv", &venv) } else { CheckItem::new("warn", "venv", "venvLater", "not made yet (the rebuild makes it)") });
            }
            // builds that can be played but not rebuilt from here (the
            // toolkit checkout the port was developed in): nothing wrong
            Some(_) => add(CheckItem::new("info", "rebuild", "rebuildElsewhere", "use the resistance-recomp checkout to rebuild")),
            None => add(CheckItem::new("warn", "rebuild", "noSources", "no port folder yet: Install the game downloads the sources").path(&src_dir())),
        }
        p.need_gb = need_gb(b.as_ref());
        let measured = root.clone().unwrap_or_else(data_root);
        let at = if measured.exists() { measured } else { home() };
        p.free_gb = free_gb(&at);
        if let Some(gb) = p.free_gb {
            add(if p.need_gb == 0 {
                CheckItem::new("ok", "disk", "diskEnough", format!("{gb} GB free: enough")).var("free", gb)
            } else if gb >= p.need_gb {
                CheckItem::new("ok", "disk", "diskNeed", format!("{gb} GB free; the install needs about {} GB", p.need_gb)).var("free", gb).var("need", p.need_gb)
            } else {
                CheckItem::new("warn", "disk", "diskLow", format!("only {gb} GB free; the install needs about {} GB", p.need_gb)).var("free", gb).var("need", p.need_gb)
            });
        }
        if let Some(b) = &b {
            p.builds_ok = b.ok;
            add(if b.ok {
                CheckItem::at("ok", "builds", Path::new(&b.root))
            } else {
                CheckItem::new("warn", "builds", "buildsMissing", format!("missing: {}", b.missing.join(", "))).var("missing", b.missing.join(", "))
            });
        }
        p.ok = !p.checks.iter().any(|c| c.level == "fail");
        p
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
fn install_clt() -> Result<(), String> {
    Command::new("xcode-select")
        .arg("--install")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("xcode-select: {e}"))
}

/// Stream a child's stdout and stderr to the window as `<event>-line`, and
/// its exit code as `<event>-exit`; `slot` holds its pid while it runs.
fn stream_child(app: &AppHandle, child: std::process::Child, event: &'static str, which: fn(&AppState) -> &Mutex<Option<u32>>) {
    stream_child_logged(app, child, event, which, None, None)
}

/// Where a child's output lines go: the window's events, or the headless
/// `--prepare`'s standard output (recomplauncher.rs).
pub(crate) type LineSink = std::sync::Arc<dyn Fn(&str) + Send + Sync>;

/// Read a child's stdout and stderr line by line into `sink` (each line also
/// written to `log`), then wait for it. The handle gives its exit code (-1
/// when it was killed by a signal) once its output has been read: the pipes
/// are given up to two seconds after it exits, in case something it started
/// still holds them.
pub(crate) fn pump_child(mut child: std::process::Child, sink: LineSink, log: Option<fs::File>) -> thread::JoinHandle<i32> {
    use std::io::Write;
    let log = std::sync::Arc::new(Mutex::new(log));
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let mut readers = 0;
    let pipes: [Option<Box<dyn Read + Send>>; 2] =
        [child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>), child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>)];
    for pipe in pipes.into_iter().flatten() {
        let (sink, log, tx) = (sink.clone(), log.clone(), tx.clone());
        readers += 1;
        thread::spawn(move || {
            for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                if let Some(f) = log.lock().unwrap().as_mut() {
                    let _ = writeln!(f, "{line}");
                }
                sink(&line);
            }
            let _ = tx.send(());
        });
    }
    thread::spawn(move || {
        let code = child.wait().map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
        for _ in 0..readers {
            if rx.recv_timeout(std::time::Duration::from_secs(2)).is_err() {
                break;
            }
        }
        code
    })
}

/// stream_child, each line also written to `log` (the install's log file);
/// `guard` (the install lock) is held until the child has exited.
fn stream_child_logged(app: &AppHandle, child: std::process::Child, event: &'static str, which: fn(&AppState) -> &Mutex<Option<u32>>, log: Option<fs::File>, guard: Option<fs::File>) {
    let a1 = app.clone();
    let line_event = format!("{event}-line");
    let done = pump_child(
        child,
        std::sync::Arc::new(move |line: &str| {
            let _ = a1.emit(&line_event, line.to_string());
        }),
        log,
    );
    let a3 = app.clone();
    thread::spawn(move || {
        let code = done.join().unwrap_or(-1);
        drop(guard);
        *which(&a3.state::<AppState>()).lock().unwrap() = None;
        let _ = a3.emit(&format!("{event}-exit"), code);
    });
}

/// Advanced > Install / rebuild: "Install the build tools", i.e.
/// tools/bootstrap_deps.sh of the port folder, streamed as setup-line /
/// setup-exit.
#[tauri::command]
fn install_deps(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    require_supported_host()?;
    if !cfg!(target_os = "macos") {
        return Err("tools/bootstrap_deps.sh is for macOS".into());
    }
    if !clt_installed() {
        return Err("install the Xcode Command Line Tools first".into());
    }
    let root = port_root(&load_settings_file()).ok_or("no port folder (Install the game on Home downloads it)")?;
    let script = root.join("tools/bootstrap_deps.sh");
    if !script.is_file() {
        return Err(format!("{}: no tools/bootstrap_deps.sh (get the new sources in Updates)", root.display()));
    }
    let mut slot = state.setup.lock().unwrap();
    if slot.is_some() {
        return Err("an install is already running".into());
    }
    let mut cmd = Command::new("/bin/zsh");
    cmd.arg(&script)
        .current_dir(&root)
        .env("PATH", tool_path())
        .env("RES_DEPS", deps_dir())
        .env("TERM", "dumb")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    own_process_group(&mut cmd);
    let child = cmd.spawn().map_err(|e| format!("zsh: {e}"))?;
    *slot = Some(child.id());
    drop(slot);
    let _ = app.emit("setup-line", format!("$ tools/bootstrap_deps.sh   # into {}", deps_dir().display()));
    stream_child(&app, child, "setup", |s| &s.setup);
    Ok(())
}

#[tauri::command]
fn cancel_setup(state: State<'_, AppState>) {
    if let Some(pid) = *state.setup.lock().unwrap() {
        kill_tree(pid, cfg!(windows));
    }
}

const STEPS: &[&str] = &["toolkit", "decrypt", "analyse", "lift", "spu", "build"];

/// The rebuild, as a zsh script run in the port folder: the build tools
/// first when `deps` (tools/bootstrap_deps.sh, when neither its folder nor
/// Homebrew has them), the toolkit, a Python venv with the toolkit's
/// requirements when there is none, then tools/rebuild.sh with the chosen
/// steps and that Python.
fn rebuild_script(steps: &[String], deps: bool) -> String {
    let want: Vec<&str> = STEPS.iter().copied().filter(|s| steps.is_empty() || steps.iter().any(|x| x == s)).collect();
    let rest: Vec<&str> = want.iter().copied().filter(|s| *s != "toolkit").collect();
    let mut sc = String::from("set -e\n");
    if deps {
        sc.push_str("echo '==> deps'\ntools/bootstrap_deps.sh\n");
    }
    if want.contains(&"toolkit") || !rest.is_empty() {
        sc.push_str("tools/rebuild.sh toolkit\n");
    }
    sc.push_str(
        "if [ ! -x .venv/bin/python ]; then\n  echo '==> venv'\n  PYB=$(command -v python3.12 || command -v python3)\n  \"$PYB\" -m venv .venv\n  .venv/bin/pip install --quiet -r ps3recomp/requirements.txt\nfi\n",
    );
    if !rest.is_empty() {
        sc.push_str(&format!("PY=$PWD/.venv/bin/python exec tools/rebuild.sh {}\n", rest.join(" ")));
    } else {
        sc.push_str("echo '==> done'\n");
    }
    sc
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebuildRequest {
    steps: Vec<String>,
    jobs: Option<u32>,
}

#[tauri::command]
fn start_rebuild(app: AppHandle, state: State<'_, AppState>, req: RebuildRequest) -> Result<(), String> {
    require_supported_host()?;
    refuse_while_updating(&state)?;
    if let Some(bad) = req.steps.iter().find(|s| !STEPS.contains(&s.as_str())) {
        return Err(format!("{bad}: not a step ({})", STEPS.join(" ")));
    }
    if state.game.lock().unwrap().is_some() {
        return Err("the game is running: close it before rebuilding".into());
    }
    let mut slot = state.rebuild.lock().unwrap();
    if slot.is_some() {
        return Err("a rebuild is already running".into());
    }
    let st = load_settings_file();
    let root = port_root(&st).ok_or("no port folder (pick the resistance-recomp checkout)")?;
    if !root.join("tools/rebuild.sh").is_file() {
        return Err(format!("{}: no tools/rebuild.sh (pick the resistance-recomp checkout)", root.display()));
    }
    let mut cmd = if cfg!(windows) {
        // untested: tools/rebuild.sh is a zsh script (MSYS2's zsh on PATH)
        let zsh = find_in_path("zsh").ok_or("the rebuild needs zsh on PATH on Windows (e.g. MSYS2); the Windows rebuild is untested")?;
        let mut c = Command::new(zsh);
        c.args(["-c", &rebuild_script(&req.steps, false)]);
        c
    } else {
        // the build tools, when neither tools/bootstrap_deps.sh nor Homebrew has them
        let deps = !deps_ready(&deps_dir()) && !brew_complete() && root.join("tools/bootstrap_deps.sh").is_file();
        let mut c = Command::new("/usr/bin/nice");
        c.args(["-n", "10", "/bin/zsh", "-c", &rebuild_script(&req.steps, deps)]);
        c
    };
    cmd.current_dir(&root)
        .env("PATH", tool_path())
        .env("RES_DEPS", deps_dir())
        .env("TERM", "dumb")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(d) = find_disc(&st) {
        cmd.env("PS3_VFS_ROOT", d);
    }
    if let Some(j) = req.jobs.filter(|j| *j > 0) {
        cmd.env("JOBS", j.to_string());
    }
    own_process_group(&mut cmd);
    let child = cmd.spawn().map_err(|e| format!("zsh: {e}"))?;
    *slot = Some(child.id());
    drop(slot);
    let _ = app.emit("rebuild-line", format!("$ cd {} && tools/rebuild.sh {}", root.display(), if req.steps.is_empty() { STEPS.join(" ") } else { req.steps.join(" ") }));
    stream_child(&app, child, "rebuild", |s| &s.rebuild);
    Ok(())
}

#[tauri::command]
fn cancel_rebuild(state: State<'_, AppState>) {
    if let Some(pid) = *state.rebuild.lock().unwrap() {
        kill_tree(pid, cfg!(windows));
    }
}

/// "Install the game" (Home), as one zsh script whose stages are announced as
/// "==> <stage>" lines (src/progress.ts): the sources when there is no
/// checkout (`root` None: cloned into `src`, at the tag of this launcher's
/// version when there is one, on the default branch so Updates can pull),
/// then tools/install.sh (deps, toolkit, venv, decrypt, analyse, lift, spu,
/// build), which skips what an earlier run finished. A checkout from before
/// tools/install.sh gets tools/bootstrap_deps.sh (when it has it) and the
/// full rebuild.
fn install_script(root: Option<&Path>, src: &Path, url: &str, tag: &str) -> String {
    let mut sc = String::from("set -e\n");
    let dir = match root {
        Some(r) => s(r),
        None => {
            let q = shell_quote(&s(src));
            sc.push_str("echo '==> sources (the port, from GitHub)'\n");
            sc.push_str(&format!("SRC={q}\n"));
            sc.push_str("if [ ! -f \"$SRC/tools/install.sh\" ]; then\n");
            sc.push_str("  rm -rf \"$SRC.partial\"\n  mkdir -p \"${SRC:h}\"\n");
            sc.push_str(&format!(
                "  git clone --quiet {} \"$SRC.partial\" || {{ echo 'ERROR: could not download the port'\"'\"'s sources. Check the internet connection and try again.' >&2; exit 1; }}\n",
                shell_quote(url)
            ));
            sc.push_str(&format!(
                "  br=$(git -C \"$SRC.partial\" rev-parse --abbrev-ref HEAD)\n  git -C \"$SRC.partial\" checkout --quiet -B \"$br\" {t} 2>/dev/null && echo \"  at {t}\" || echo '  (no tag {t}: the newest sources)'\n",
                t = shell_quote(tag)
            ));
            sc.push_str("  rm -rf \"$SRC\"\n  mv \"$SRC.partial\" \"$SRC\"\nfi\n");
            sc.push_str("cd \"$SRC\"\nexec tools/install.sh\n");
            return sc;
        }
    };
    sc.push_str(&format!("cd {}\n", shell_quote(&dir)));
    let r = Path::new(&dir);
    if r.join("tools/install.sh").is_file() {
        sc.push_str("exec tools/install.sh\n");
    } else {
        sc.push_str(rebuild_script(&[], r.join("tools/bootstrap_deps.sh").is_file()).trim_start_matches("set -e\n"));
    }
    sc
}

/// Why "Install the game" cannot start. The window explains Clt and NoDisc
/// itself; the headless `--prepare` (recomplauncher.rs) turns each into an
/// exit code.
#[derive(Debug)]
pub(crate) enum InstallError {
    /// Apple's Command Line Tools are missing (their installer is the player's).
    Clt,
    /// No disc to decrypt from, and the ELFs are not there yet.
    NoDisc,
    Other(String),
}

impl InstallError {
    /// The window's form: "clt" and "no-disc" as codes, else the message.
    fn window(self) -> String {
        match self {
            InstallError::Clt => "clt".into(),
            InstallError::NoDisc => "no-disc".into(),
            InstallError::Other(m) => m,
        }
    }
}

/// "Install the game", ready to start: the zsh script's command (stdout and
/// stderr piped, not in a process group of its own yet), the port folder it
/// works in, its first log line (already in the log file), and the log file.
pub(crate) struct InstallJob {
    cmd: Command,
    at: PathBuf,
    first: String,
    log: Option<fs::File>,
}

/// Everything "Install the game" does before its script starts, shared by the
/// window's button and the headless `--prepare`: the Command Line Tools, the
/// port folder (a hand-set one that cannot be rebuilt gives way to the
/// install's own sources), the disc (the remembered image attached when it
/// is not mounted), and the script (see install_script).
pub(crate) fn install_job() -> Result<InstallJob, InstallError> {
    if !cfg!(target_os = "macos") {
        return Err(InstallError::Other("the one-click install is for macOS; use Advanced > Install / rebuild".into()));
    }
    if !clt_installed() {
        return Err(InstallError::Clt);
    }
    let mut st = load_settings_file();
    let root = port_root(&st).filter(|r| r.join("tools/rebuild.sh").is_file());
    if root.is_none() && !st.port_dir.trim().is_empty() {
        // a port folder set by hand that cannot be rebuilt: the install's own
        // sources take over (found on their own once the setting is empty)
        st.port_dir.clear();
        write_settings(&st).map_err(InstallError::Other)?;
    }
    let b = root.as_deref().map(resolve_builds);
    let elfs = b.as_ref().is_some_and(|b| b.eboot_elf.is_some() && b.game_elf.is_some());
    let disc = match find_disc(&st) {
        Some(d) => Some(d),
        None if !st.disc_image.trim().is_empty() => attach_image(Path::new(st.disc_image.trim())).ok(),
        None => None,
    };
    if disc.is_none() && !elfs {
        return Err(InstallError::NoDisc);
    }
    let data = data_root();
    fs::create_dir_all(&data).map_err(|e| InstallError::Other(format!("{}: {e}", data.display())))?;
    let script = install_script(root.as_deref(), &src_dir(), &sources_url(), &format!("v{}", env!("CARGO_PKG_VERSION")));
    let mut cmd = Command::new("/usr/bin/nice");
    cmd.args(["-n", "10", "/bin/zsh", "-c", &script])
        .current_dir(&data)
        .env("PATH", tool_path())
        .env("RES_DEPS", deps_dir())
        .env("TERM", "dumb")
        .env("GIT_TERMINAL_PROMPT", "0")
        // the scripts find their checkout from where they are
        .env_remove("RES_ROOT")
        .env_remove("PS3RECOMP_DIR")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(d) = &disc {
        cmd.env("PS3_VFS_ROOT", d);
    }
    if st.jobs > 0 {
        cmd.env("JOBS", st.jobs.to_string());
    }
    let log = fs::File::create(install_log_path()).ok();
    let at = root.unwrap_or_else(src_dir);
    let first = format!("$ tools/install.sh   # in {}", at.display());
    if let Some(mut f) = log.as_ref().and_then(|f| f.try_clone().ok()) {
        use std::io::Write;
        let _ = writeln!(f, "{first}");
    }
    Ok(InstallJob { cmd, at, first, log })
}

/// "Install the game": everything from the disc to Play, in one go (see
/// install_job). Errors the window explains itself: "clt" (the Command
/// Line Tools are missing), "no-disc". Returns the port folder it works in.
#[tauri::command]
fn start_install(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    require_supported_host()?;
    refuse_while_updating(&state)?;
    if !cfg!(target_os = "macos") {
        return Err("the one-click install is for macOS; use Advanced > Install / rebuild".into());
    }
    if state.game.lock().unwrap().is_some() {
        return Err("the game is running: close it first".into());
    }
    let mut slot = state.rebuild.lock().unwrap();
    if slot.is_some() {
        return Err("an install or a rebuild is already running".into());
    }
    // an install started outside this window (RecompLauncher's setup, a
    // second launcher); a lock that cannot be taken at all does not stop it
    let lock = match install_lock() {
        Ok(Some(f)) => Some(f),
        Ok(None) => return Err("an install is already running outside this window (RecompLauncher's setup?): wait for it to finish".into()),
        Err(_) => None,
    };
    let InstallJob { mut cmd, at, first, log } = install_job().map_err(InstallError::window)?;
    own_process_group(&mut cmd);
    let child = cmd.spawn().map_err(|e| format!("zsh: {e}"))?;
    *slot = Some(child.id());
    drop(slot);
    let _ = app.emit("rebuild-line", first);
    stream_child_logged(&app, child, "rebuild", |s| &s.rebuild, log, lock);
    Ok(s(&at))
}

// ---------------------------------------------------------------------------
// Updates (update.rs)
// ---------------------------------------------------------------------------

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    sources: update::Sources,
    /// The newest commit touching the port is newer than the EBOOT build.
    rebuild_needed: bool,
    built_at: u64,
    /// The launcher (latest.json, the install, "updated from").
    app: update::AppUpdate,
}

#[tauri::command]
async fn check_updates(app: AppHandle, online: bool) -> UpdateInfo {
    tauri::async_runtime::spawn_blocking(move || {
        let st = load_settings_file();
        let root = port_root(&st);
        let b = root.as_deref().map(resolve_builds);
        let sources = root.as_deref().map(|r| update::sources(r, online)).unwrap_or_default();
        let built_at = b.as_ref().map_or(0, |b| b.built_at);
        let rebuild_needed = built_at > 0 && sources.sources_time > built_at;
        let app = update::check_app(&app, online, built_at);
        UpdateInfo { sources, rebuild_needed, built_at, app }
    })
    .await
    .unwrap_or_default()
}

/// `git pull --ff-only` in the port folder, output piped (Updates' "Get the
/// new sources", and the headless `--prepare` on the install's own sources).
pub(crate) fn pull_command(root: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(root)
        .args(["pull", "--ff-only"])
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("PATH", tool_path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

/// "Get the new sources": git pull --ff-only in the port folder, streamed as
/// rebuild-line / rebuild-exit (the Install tab's log).
#[tauri::command]
fn pull_sources(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    refuse_while_updating(&state)?;
    let mut slot = state.rebuild.lock().unwrap();
    if slot.is_some() {
        return Err("a rebuild is running".into());
    }
    let root = port_root(&load_settings_file()).ok_or("no port folder")?;
    let mut cmd = pull_command(&root);
    own_process_group(&mut cmd);
    let child = cmd.spawn().map_err(|e| format!("git: {e}"))?;
    *slot = Some(child.id());
    drop(slot);
    let _ = app.emit("rebuild-line", format!("$ git -C {} pull --ff-only", root.display()));
    stream_child(&app, child, "rebuild", |s| &s.rebuild);
    Ok(())
}

// ---------------------------------------------------------------------------
// Links, files, the log
// ---------------------------------------------------------------------------

const URL_ALLOWED: &[&str] = &["https://rpcs3.net/", "https://brew.sh/", "https://github.com/erikomis/resistance-recomp", "https://github.com/sp00nznet/ps3recomp", "https://psrewired.com/"];

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if !URL_ALLOWED.iter().any(|p| url.starts_with(p)) || url.chars().any(|c| c.is_whitespace() || c == '"') {
        return Err(format!("{url}: not a link this app opens"));
    }
    if cfg!(windows) {
        return Command::new("rundll32").arg("url.dll,FileProtocolHandler").arg(&url).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    Command::new("open").arg(&url).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Explorer: open a folder, or select a file in its folder (Windows).
#[cfg(windows)]
fn reveal_windows(p: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("explorer");
    if p.is_dir() {
        cmd.arg(p);
    } else if p.exists() {
        // raw: explorer parses /select, itself; a '"' (never in a Windows
        // path) could not be quoted, so it is refused
        let t = p.display().to_string();
        if t.contains('"') {
            return Err(format!("{t}: not a path"));
        }
        cmd.raw_arg(format!("/select,\"{t}\""));
    } else {
        cmd.arg(p.parent().unwrap_or(p));
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Show a folder or file in Finder (never opens an .app: that would run it).
#[tauri::command]
fn reveal(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    #[cfg(windows)]
    {
        reveal_windows(&p)
    }
    #[cfg(not(windows))]
    reveal_unix(p)
}

#[cfg(not(windows))]
fn reveal_unix(p: PathBuf) -> Result<(), String> {
    if !p.exists() {
        let parent = p.parent().map(Path::to_path_buf).unwrap_or(p);
        return Command::new("open").arg(&parent).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    if p.is_dir() && p.extension().is_none_or(|e| e != "app") {
        Command::new("open").arg(&p).spawn().map(|_| ()).map_err(|e| e.to_string())
    } else {
        Command::new("open").arg("-R").arg(&p).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
}

fn game_log_path(state: &State<'_, AppState>) -> PathBuf {
    state.last_log.lock().unwrap().clone().unwrap_or_else(|| play_dir(&load_settings_file()).join("last.log"))
}

fn tail_text(data: &[u8], cut: bool) -> String {
    let start = if cut { data.iter().position(|&b| b == b'\n').map_or(0, |i| i + 1) } else { 0 };
    String::from_utf8_lossy(&data[start..]).into_owned()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLog {
    path: String,
    exists: bool,
    size: u64,
    modified: u64,
    text: String,
    truncated: bool,
}

#[tauri::command]
fn read_game_log(state: State<'_, AppState>, max_bytes: Option<u64>) -> Result<GameLog, String> {
    let path = game_log_path(&state);
    let max = max_bytes.unwrap_or(512 * 1024).clamp(4 * 1024, 8 * 1024 * 1024);
    let mut out = GameLog { path: s(&path), exists: false, size: 0, modified: 0, text: String::new(), truncated: false };
    let Ok(mut f) = fs::File::open(&path) else { return Ok(out) };
    let meta = f.metadata().map_err(|e| e.to_string())?;
    out.exists = true;
    out.size = meta.len();
    out.modified = mtime(&path);
    out.truncated = meta.len() > max;
    if out.truncated {
        f.seek(SeekFrom::Start(meta.len() - max)).map_err(|e| e.to_string())?;
    }
    let mut buf = Vec::with_capacity(max.min(meta.len()) as usize);
    f.take(max).read_to_end(&mut buf).map_err(|e| e.to_string())?;
    out.text = tail_text(&buf, out.truncated);
    Ok(out)
}

#[tauri::command]
fn log_paths(state: State<'_, AppState>) -> BTreeMap<String, String> {
    let st = load_settings_file();
    let mut m = BTreeMap::new();
    m.insert("game".into(), s(&game_log_path(&state)));
    m.insert("play".into(), s(&play_dir(&st)));
    m.insert("saves".into(), s(&save_locations(&st).port_savedata));
    m.insert("data".into(), s(&data_root()));
    m.insert("install".into(), s(&install_log_path()));
    m
}

// ---------------------------------------------------------------------------
// Shader cache
// ---------------------------------------------------------------------------

fn dir_usage(dir: &Path) -> (u64, u64) {
    let (mut bytes, mut files) = (0u64, 0u64);
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            match e.metadata() {
                Ok(m) if m.is_dir() => {
                    let (b, f) = dir_usage(&e.path());
                    bytes += b;
                    files += f;
                }
                Ok(m) => {
                    bytes += m.len();
                    files += 1;
                }
                Err(_) => {}
            }
        }
    }
    (bytes, files)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderCacheInfo {
    path: Option<String>,
    bytes: u64,
    files: u64,
}

fn shader_cache_info_now() -> ShaderCacheInfo {
    match shader_cache_dir(&load_settings_file()) {
        Some(d) => {
            let (bytes, files) = dir_usage(&d);
            ShaderCacheInfo { path: Some(s(&d)), bytes, files }
        }
        None => ShaderCacheInfo { path: None, bytes: 0, files: 0 },
    }
}

#[tauri::command]
fn shader_cache_info() -> ShaderCacheInfo {
    shader_cache_info_now()
}

#[tauri::command]
fn shader_cache_clear(state: State<'_, AppState>) -> Result<ShaderCacheInfo, String> {
    if state.game.lock().unwrap().is_some() || process_running(GAME_EXE) {
        return Err("the game is running: close it before clearing the shader cache".into());
    }
    if let Some(d) = shader_cache_dir(&load_settings_file()) {
        if d.exists() {
            fs::remove_dir_all(&d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
    }
    Ok(shader_cache_info_now())
}

// ---------------------------------------------------------------------------
// Debug mode (report.rs)
// ---------------------------------------------------------------------------

fn session_event(dir: &Path, kind: &str, detail: &str) {
    let ms = now_ms();
    let _ = report::event(dir, ms, &utc_text(ms / 1000), kind, detail);
}

fn spawn_ticker(app: AppHandle, dir: PathBuf) {
    thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_secs(2));
        let live = matches!(app.state::<AppState>().report.lock().unwrap().as_ref(), Some((d, true)) if *d == dir);
        if !live {
            break;
        }
        session_event(&dir, "tick", "");
    });
}

fn session_exit(dir: &Path, status: Option<std::process::ExitStatus>, started: u64) {
    #[cfg(unix)]
    let signal = {
        use std::os::unix::process::ExitStatusExt;
        status.and_then(|s| s.signal())
    };
    #[cfg(not(unix))]
    let signal: Option<i32> = None;
    let code = status.and_then(|s| s.code());
    let detail = match (code, signal) {
        (Some(c), _) => format!("code={c}"),
        (None, Some(sig)) => format!("signal={sig}"),
        _ => "unknown".into(),
    };
    session_event(dir, "exit", &detail);
    let stopped = fs::read_to_string(dir.join(report::EVENTS)).is_ok_and(|t| t.contains("\tstop\t"));
    let crashed = signal.is_some() || code.is_some_and(|c| c != 0);
    if !crashed || stopped || !cfg!(target_os = "macos") {
        return;
    }
    let diag = home().join("Library/Logs/DiagnosticReports");
    for _ in 0..20 {
        if let Some(p) = report::find_crash_report(&diag, &[GAME_EXE, HOST_EXE], started) {
            if fs::copy(&p, dir.join(report::CRASH)).is_ok() {
                session_event(dir, "crash-report", &format!("file={}", p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()));
            }
            return;
        }
        thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn command_line(program: &str, args: &[&str]) -> String {
    command_stdout(Command::new(program).args(args)).map(|s| s.trim().to_string()).unwrap_or_default()
}

/// UTC "YYYY-MM-DD HH:MM:SS".
fn utc_text(secs: u64) -> String {
    let st = saves::stamp(secs);
    format!("{}-{}-{} {}:{}:{}", &st[0..4], &st[4..6], &st[6..8], &st[9..11], &st[11..13], &st[13..15])
}

fn system_info(b: &Builds, disc: &Path, env: &[(String, String)], argv: &[String], now: u64) -> String {
    let mut t = String::new();
    let mut line = |k: &str, v: &str| {
        if !v.is_empty() {
            t.push_str(&format!("{k}: {v}\n"));
        }
    };
    line("report", &utc_text(now));
    line("launcher", env!("CARGO_PKG_VERSION"));
    line("os", &format!("{} {}", std::env::consts::OS, std::env::consts::ARCH));
    if cfg!(target_os = "macos") {
        line("macos", &format!("{} ({})", command_line("sw_vers", &["-productVersion"]), command_line("sw_vers", &["-buildVersion"])));
        line("model", &command_line("sysctl", &["-n", "hw.model"]));
        line("cpu", &command_line("sysctl", &["-n", "machdep.cpu.brand_string"]));
        let mem = command_line("sysctl", &["-n", "hw.memsize"]).parse::<u64>().unwrap_or(0);
        if mem > 0 {
            line("memory", &format!("{} GB", mem >> 30));
        }
        let gpus = command_stdout(Command::new("/usr/sbin/system_profiler").args(["SPDisplaysDataType", "-json"]))
            .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
            .map(|v| {
                v["SPDisplaysDataType"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|g| g["sppci_model"].as_str().map(String::from))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        line("gpu", &gpus);
    }
    let v = read_sfo(&disc.join("PS3_GAME/PARAM.SFO")).unwrap_or_default();
    line("disc", &format!("{} {} {}", v.get("TITLE_ID").cloned().unwrap_or_default(), v.get("TITLE").cloned().unwrap_or_default(), v.get("APP_VER").cloned().unwrap_or_default()));
    line("port folder", &b.root);
    line("built", &utc_text(b.built_at));
    line("commit", &command_line("git", &["-C", &b.root, "rev-parse", "--short", "HEAD"]));
    line("campaign v1.7 build", if b.v17 { "yes" } else { "no" });
    t.push_str("\ncommand:\n  ");
    t.push_str(&argv.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" "));
    t.push_str("\n\nenvironment:\n");
    for (k, v) in env {
        t.push_str(&format!("  {k}={}\n", shell_quote(v)));
    }
    let home = s(&home());
    if home.len() > 1 {
        t = t.replace(&home, "~");
    }
    t
}

fn current_report(state: &State<'_, AppState>) -> Option<(PathBuf, bool)> {
    let r = state.report.lock().unwrap().clone();
    r.or_else(|| report::latest(&reports_dir()).map(|d| (d, false)))
}

#[tauri::command]
fn report_info(state: State<'_, AppState>) -> Option<report::Info> {
    current_report(&state).map(|(d, live)| report::info(&d, live))
}

#[tauri::command]
fn report_capture(state: State<'_, AppState>) -> Result<report::Info, String> {
    let game = state.game.lock().unwrap().is_some();
    match state.report.lock().unwrap().clone() {
        Some((d, true)) if game => {
            let ms = now_ms();
            report::capture(&d, &utc_text(ms / 1000), ms)?;
            Ok(report::info(&d, true))
        }
        _ => Err("the game was not started in debug mode".into()),
    }
}

fn report_dir_of(state: &State<'_, AppState>, id: Option<&str>) -> Result<(PathBuf, bool), String> {
    let cur = current_report(state);
    match id {
        Some(id) => {
            let d = report::by_id(&reports_dir(), id)?;
            let live = cur.is_some_and(|(c, live)| live && c == d);
            Ok((d, live))
        }
        None => cur.ok_or_else(|| "no report yet: start the game in debug mode first".into()),
    }
}

#[tauri::command]
async fn report_list(state: State<'_, AppState>) -> Result<Vec<report::Info>, String> {
    let cur = current_report(&state);
    tauri::async_runtime::spawn_blocking(move || {
        report::list(&reports_dir(), 40)
            .into_iter()
            .map(|d| {
                let live = cur.as_ref().is_some_and(|(c, live)| *live && *c == d);
                report::info(&d, live)
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn report_triage(state: State<'_, AppState>, id: String) -> Result<report::Info, String> {
    let (dir, live) = report_dir_of(&state, Some(&id))?;
    tauri::async_runtime::spawn_blocking(move || {
        report::summarise(&dir)?;
        Ok(report::info(&dir, live))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn report_zip(state: State<'_, AppState>, id: Option<String>, notes: Option<String>) -> Result<report::Info, String> {
    let (dir, live) = report_dir_of(&state, id.as_deref())?;
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(n) = notes.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
            fs::write(dir.join(report::NOTES), format!("{n}\n")).map_err(|e| e.to_string())?;
        }
        let _ = report::summarise(&dir);
        report::pack(&dir, &report::zip_path(&dir), &s(&home()))?;
        Ok(report::info(&dir, live))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn report_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let (_, live) = report_dir_of(&state, Some(&id))?;
    if live {
        return Err("that session is still running".into());
    }
    report::delete(&reports_dir(), &id)
}

// ---------------------------------------------------------------------------
// Saves (saves.rs)
// ---------------------------------------------------------------------------

fn game_busy(state: &AppState) -> bool {
    state.game.lock().unwrap().is_some() || process_running(GAME_EXE) || process_running(LEGACY_EXE)
}

fn with_saves<T>(state: &State<'_, AppState>, f: impl FnOnce(&saves::Locations) -> Result<T, String>) -> Result<T, String> {
    let game = state.game.lock().unwrap();
    if game.is_some() || process_running(GAME_EXE) || process_running(LEGACY_EXE) {
        return Err("the game is running: close it before changing saves".into());
    }
    let r = f(&save_locations(&load_settings_file()));
    drop(game);
    r
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavesOverview {
    port: Vec<Slot>,
    rpcs3: Vec<Slot>,
    rpcs3_users: Vec<String>,
    rpcs3_user: String,
    backups: Vec<BackupMeta>,
    port_dir: String,
    port_usrdir: String,
    rpcs3_dir: String,
    backups_dir: String,
    game_running: bool,
    rpcs3_running: bool,
}

#[tauri::command]
async fn saves_overview(state: State<'_, AppState>, user: Option<String>) -> Result<SavesOverview, String> {
    let running = game_busy(&state);
    tauri::async_runtime::spawn_blocking(move || {
        let loc = save_locations(&load_settings_file());
        let users = saves::rpcs3_users(&loc);
        let user = user.filter(|u| users.contains(u)).or_else(|| users.first().cloned()).unwrap_or_else(|| saves::PORT_USER.into());
        SavesOverview {
            port: saves::list_slots(&loc, Side::Port, "", true),
            rpcs3: if users.is_empty() { Vec::new() } else { saves::list_slots(&loc, Side::Rpcs3, &user, true) },
            rpcs3_users: users,
            rpcs3_user: user.clone(),
            backups: saves::list_backups(&loc),
            port_dir: s(&loc.port_savedata),
            port_usrdir: s(&loc.port_usrdir),
            rpcs3_dir: s(&loc.rpcs3_savedata(&user)),
            backups_dir: s(&loc.backups),
            game_running: running,
            rpcs3_running: process_running("rpcs3") || process_running("RPCS3"),
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn saves_backup(state: State<'_, AppState>, side: Side, user: String, slot: Option<String>) -> Result<Option<BackupMeta>, String> {
    with_saves(&state, |loc| {
        let one = slot.map(|s| vec![s]);
        saves::create_backup(loc, side, &user, one.as_deref(), "manual", now_secs())
    })
}

#[tauri::command]
async fn saves_restore(state: State<'_, AppState>, id: String, slot: Option<String>) -> Result<saves::Report, String> {
    with_saves(&state, |loc| saves::restore_backup(loc, &id, slot.as_deref(), now_secs()))
}

#[tauri::command]
async fn saves_delete_backup(state: State<'_, AppState>, id: String) -> Result<(), String> {
    with_saves(&state, |loc| saves::delete_backup(loc, &id))
}

#[tauri::command]
async fn saves_copy(state: State<'_, AppState>, from: Side, to: Side, user: String, slot: String) -> Result<saves::Report, String> {
    with_saves(&state, |loc| saves::copy_slot(loc, from, to, &user, &slot, now_secs()))
}

#[tauri::command]
async fn saves_delete_slot(state: State<'_, AppState>, side: Side, user: String, slot: String) -> Result<BackupMeta, String> {
    with_saves(&state, |loc| saves::delete_slot(loc, side, &user, &slot, now_secs()))
}

#[tauri::command]
async fn saves_export_zip(state: State<'_, AppState>, slots: Option<Vec<String>>, dest: String) -> Result<String, String> {
    let mut dest = PathBuf::from(dest.trim());
    if dest.as_os_str().is_empty() {
        return Err("no file chosen".into());
    }
    if !dest.extension().is_some_and(|e| e.eq_ignore_ascii_case("zip")) {
        dest.as_mut_os_string().push(".zip");
    }
    with_saves(&state, |loc| saves::export_zip(loc, slots.as_deref(), &dest, now_secs()))?;
    Ok(s(&dest))
}

#[tauri::command]
async fn saves_import_zip(state: State<'_, AppState>, path: String) -> Result<saves::Report, String> {
    with_saves(&state, |loc| saves::import_zip(loc, Path::new(path.trim()), now_secs()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // RecompLauncher's way in (recomplauncher.rs): `--prepare` and, started
    // by it (RECOMPLAUNCHER=1), `--play` never open the window, and are
    // decided here, before Tauri (its event loop, the updater) starts at all.
    let args: Vec<String> = std::env::args().collect();
    let state = AppState::default();
    match recomplauncher::mode(&args, std::env::var("RECOMPLAUNCHER").is_ok_and(|v| v == "1")) {
        Ok(recomplauncher::Mode::Prepare { game }) => std::process::exit(recomplauncher::prepare(game.as_deref()).code()),
        Ok(recomplauncher::Mode::PlayHeadless { game }) => std::process::exit(recomplauncher::play(game.as_deref())),
        Ok(recomplauncher::Mode::PlayOnlineHeadless { game }) => std::process::exit(recomplauncher::play_online(game.as_deref())),
        Ok(recomplauncher::Mode::Window { game, online }) => {
            // `--play [--play-online] --game <disc>` from another program:
            // that disc, remembered
            if let Some(g) = game {
                if let Err(e) = recomplauncher::use_disc(&g) {
                    eprintln!("--game {}: {}", g.display(), e.message());
                }
            }
            *state.online_request.lock().unwrap() = online;
        }
        Err(e) => {
            eprintln!("{e}\n{}", recomplauncher::USAGE);
            std::process::exit(recomplauncher::Exit::Usage.code());
        }
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        .setup(|app| {
            sync_app_icon(app.handle());
            let args: Vec<String> = std::env::args().collect();
            let st = app.state::<AppState>();
            // online play asked for: not the offline Play as well
            let online = *st.online_request.lock().unwrap();
            *st.play_request.lock().unwrap() = !online && play_shortcut::requested(args.iter().cloned());
            *st.exit_with_game.lock().unwrap() = play_shortcut::exit_requested(&args);
            *st.start_view.lock().unwrap() = play_shortcut::start_view(&args);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_input_map,
            save_input_map,
            reset_input_map,
            list_screens,
            list_mods,
            get_status,
            find_port_dirs,
            builds_at,
            disc_status,
            mount_image,
            eject_disc,
            move_update_files,
            check_updates,
            pull_sources,
            play_online,
            build_online_menus,
            update::install_update,
            update::restart_app,
            check_prereqs,
            install_clt,
            install_deps,
            start_install,
            cancel_setup,
            open_url,
            start_rebuild,
            cancel_rebuild,
            launch_preview,
            play,
            stop_game,
            reveal,
            log_paths,
            read_game_log,
            shader_cache_info,
            shader_cache_clear,
            saves_overview,
            saves_backup,
            saves_restore,
            saves_delete_backup,
            saves_copy,
            saves_delete_slot,
            saves_export_zip,
            saves_import_zip,
            report_info,
            report_list,
            report_triage,
            report_capture,
            report_zip,
            report_delete,
            finder_icon,
            take_play_request,
            take_online_request,
            take_start_view,
            create_play_shortcut,
            steam_status,
            steam_add,
            steam_remove,
            steam_quit
        ])
        .build(tauri::generate_context!())
        .expect("error while running the launcher")
        .run(|app, ev| {
            // Quitting stops an install or a rebuild (its own process group,
            // by pid); "Install the game" continues where it stopped next time.
            if let tauri::RunEvent::Exit = ev {
                let st = app.state::<AppState>();
                for slot in [&st.rebuild, &st.setup] {
                    if let Some(pid) = *slot.lock().unwrap() {
                        kill_tree(pid, false);
                    }
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("rl-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn base64_matches_rfc4648() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn env_lines() {
        let v = parse_env_lines("# c\nA=1\nexport B=\"x y\"\nbad key=2\n\nC=");
        assert_eq!(v, vec![("A".into(), "1".into()), ("B".into(), "x y".into()), ("C".into(), "".into())]);
    }

    #[test]
    fn plist_values() {
        let xml = "<dict><key>image-path</key>\n\t<string>/a/b &amp; c.iso</string><key>system-entities</key><array>\
                   <dict><key>mount-point</key><string>/Volumes/RESISTANCE_1</string></dict></array></dict>";
        assert_eq!(plist_strings(xml, "image-path"), vec!["/a/b & c.iso".to_string()]);
        assert_eq!(plist_strings(xml, "mount-point"), vec!["/Volumes/RESISTANCE_1".to_string()]);
    }

    #[test]
    fn builds_and_disc_layout() {
        let root = tmp("builds");
        let b = resolve_builds(&root);
        assert!(!b.ok && b.missing.len() == 5 && b.rebuild_sh.is_none());
        for f in ["build/BCUS98107/Resistance", "build/BCUS98107/MyGameRecomp", "build/BCUS98107_game/ResistanceRecomp", "game/BCUS98107/EBOOT.elf", "game/BCUS98107/modules/game.elf"] {
            let p = root.join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, b"x").unwrap();
        }
        let b = resolve_builds(&root);
        assert!(b.ok && b.legacy && !b.v17, "{:?}", b.missing);
        for f in ["build/BCUS98107_game_v17/ResistanceRecomp", "game/BCUS98107/modules_v17/game.elf"] {
            let p = root.join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, b"x").unwrap();
        }
        assert!(resolve_builds(&root).v17);

        let mut st = Settings::default();
        st.window_size = "1920x1080".into();
        st.keep_awake = false;
        let argv = launch_argv(&st, &resolve_builds(&root)).unwrap();
        assert_eq!(argv[0], s(&root.join("build/BCUS98107/Resistance")));
        assert_eq!(&argv[1..4], &["--size".to_string(), "1920x1080".into(), "--".into()]);
        assert!(argv[4].ends_with("build/BCUS98107/MyGameRecomp") && argv[5].ends_with("EBOOT.elf"));

        let disc = root.join("disc");
        fs::create_dir_all(disc.join("PS3_GAME")).unwrap();
        assert!(disc_root_of(&disc).is_none());
        fs::write(disc.join("PS3_GAME/PARAM.SFO"), saves::psf_write(&[])).unwrap();
        assert_eq!(disc_root_of(&disc), Some(disc.clone()));
        assert_eq!(disc_root_of(&disc.join("PS3_GAME")), Some(disc.clone()));

        let play = root.join("play");
        assert!(first_boot(&play) && update_files(&play).is_empty());
        let data = game_data_dir(&play).join("USRDIR/data");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("patch_06.bin"), b"x").unwrap();
        fs::write(data.join("patch_06.psarc"), b"x").unwrap();
        assert_eq!(update_files(&play), vec![data.join("patch_06.bin")]);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn online_ids_follow_the_psn_rules() {
        for ok in ["abc", "Player_01", "a-b-c", "x234567890123456"] {
            assert!(valid_online_id(ok), "{ok}");
        }
        for bad in ["", "ab", "1abc", "_abc", "abc def", "x2345678901234567", "ação", "a.b"] {
            assert!(!valid_online_id(bad), "{bad}");
        }
    }

    #[test]
    fn online_env_never_changes_the_frame_rate() {
        let mut st = Settings::default();
        st.online_id = "Player_01".into();
        st.unlock_fps = true;
        st.menu_60fps = true;
        st.campaign = "100".into();
        st.extra_env = "RES_UNLOCK_FPS=1\nPS3_VBLANK_HZ=120\nPS3_NET_ALLOW=1.2.3.4\nPS3_NP_ONLINE_ID=someone\nPS3_AA=fxaa".into();
        let e: BTreeMap<String, String> =
            online_env(&st, Path::new("/Volumes/RESISTANCE_1"), Path::new("/p/online"), &InputEnv::default()).into_iter().collect();
        for k in ["RES_UNLOCK_FPS", "RES_MENU_60FPS", "PS3_VBLANK_HZ", "RES_GAME_VERSION"] {
            assert!(!e.contains_key(k), "{k} passed online");
        }
        assert_eq!(e["PS3_HDD0_ROOT"], "/p/online/dev_hdd0");
        assert_eq!(e["PS3_OVERLAY_INI"], "/p/online/overlay.ini");
        assert_eq!(e["PS3_NET_ONLINE"], "1");
        assert_eq!(e["PS3_NP_SIGNIN"], "1");
        assert_eq!(e["PS3_NP_ONLINE_ID"], "Player_01");
        assert_eq!(e["PS3_NET_DNS"], ONLINE_DNS);
        assert_eq!(e["PS3_NET_ALLOW"], ONLINE_ALLOW);
        assert_eq!(e["PS3_NET_LOCAL_IP"], ONLINE_LOCAL_IP);
        assert_eq!(e["PS3_NET_NO_UPNP"], "1");
        assert_eq!(e["PS3_AA"], "fxaa"); // the other extra variables still apply
        assert!(!e.contains_key("PS3_MIC"), "the microphone without voice chat");
        let mut sv = st.clone();
        sv.voice_chat = true;
        sv.extra_env.push_str("\nPS3_MIC_TONE=1");
        let ev: BTreeMap<String, String> = online_env(&sv, Path::new("/d"), Path::new("/o"), &InputEnv::default()).into_iter().collect();
        assert_eq!(ev["PS3_MIC"], "1");
        assert!(!ev.contains_key("PS3_MIC_TONE"));
        // offline Play never opens the microphone
        let off: BTreeMap<String, String> = launch_env(&sv, Path::new("/d"), Path::new("/p"), &InputEnv::default()).into_iter().collect();
        assert!(!off.contains_key("PS3_MIC"));
        // one value per variable: the online ones are not doubled by the extras
        let v = online_env(&st, Path::new("/d"), Path::new("/o"), &InputEnv::default());
        assert_eq!(v.iter().filter(|(k, _)| k == "PS3_NET_ALLOW").count(), 1);
    }

    /// Play online's start, the window's and the headless --play-online's
    /// (prepare_online_launch), on a fake port folder and disc: the online
    /// folder, the disc's menus until the v1.7 ones are built, and never a
    /// frame-rate variable, whatever the settings and extra variables say.
    #[test]
    fn online_launch_runs_at_the_games_frame_rate() {
        let t = tmp("online-launch");
        let root = t.join("port");
        let touch = |p: PathBuf| {
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, b"x").unwrap();
        };
        for f in [format!("build/{TITLE_ID}/{GAME_EXE}"), format!("build/{TITLE_ID}/{HOST_EXE}"), format!("build/{TITLE_ID}_game/{GAME_EXE}"), format!("game/{TITLE_ID}/EBOOT.elf"), format!("game/{TITLE_ID}/modules/game.elf")] {
            touch(root.join(f));
        }
        let disc = t.join("disc");
        fs::create_dir_all(disc.join("PS3_GAME")).unwrap();
        let e = |k: &str, v: &str| saves::PsfEntry { key: k.into(), fmt: saves::PSF_UTF8, data: format!("{v}\0").into_bytes(), max_len: 16 };
        fs::write(disc.join("PS3_GAME/PARAM.SFO"), saves::psf_write(&[e("TITLE_ID", TITLE_ID)])).unwrap();
        let mut st = Settings::default();
        st.port_dir = s(&root);
        st.disc_path = s(&disc);
        st.play_dir = s(&t.join("play"));
        st.keep_awake = true;
        st.unlock_fps = true;
        st.menu_60fps = true;
        st.campaign = "100".into();
        st.extra_env = "RES_UNLOCK_FPS=1\nRES_MENU_60FPS=1\nPS3_VBLANK_HZ=120\nPS3_FLIP_MAX_HZ=120\nPS3_NET_ALLOW=1.2.3.4\nPS3_NP_ONLINE_ID=someone".into();
        let input = InputEnv::default();

        // no online ID: nothing is written
        assert!(matches!(prepare_online_launch(&st, &input, false), Err(LaunchError::OnlineId)));
        assert!(!t.join("play/online").exists());
        st.online_id = "Player_01".into();

        let check = |l: &OnlineLaunch| {
            let envs: BTreeMap<String, Option<String>> =
                l.cmd.get_envs().map(|(k, v)| (k.to_string_lossy().into_owned(), v.map(|v| v.to_string_lossy().into_owned()))).collect();
            for k in ["RES_UNLOCK_FPS", "RES_MENU_60FPS", "PS3_VBLANK_HZ", "PS3_FLIP_MAX_HZ", "RES_GAME_VERSION"] {
                // taken out of what the game would inherit, and never set
                assert_eq!(envs.get(k), Some(&None), "{k} reaches online play");
            }
            for (k, v) in &envs {
                if let Some(v) = v {
                    assert!(!online_forbidden(k) || k.starts_with("PS3_NET_") || k.starts_with("PS3_NP_"), "{k}={v} online");
                }
            }
            assert_eq!(envs["PS3_NET_ALLOW"].as_deref(), Some(ONLINE_ALLOW));
            assert_eq!(envs["PS3_NP_ONLINE_ID"].as_deref(), Some("Player_01"));
            assert_eq!(envs["PS3_HDD0_ROOT"].as_deref(), Some(s(&t.join("play/online/dev_hdd0")).as_str()));
            assert_eq!(l.cmd.get_current_dir(), Some(t.join("play/online").as_path()));
            assert_eq!(l.log_path, t.join("play/online/last.log"));
        };
        // the headless start keeps the Mac awake its own way
        let no_caffeinate = |l: &OnlineLaunch| assert!(!l.argv.iter().any(|a| a.contains("caffeinate")), "{:?}", l.argv);
        // first time: the disc's menus, so the game can download its update
        let l = prepare_online_launch(&st, &input, false).unwrap();
        check(&l);
        no_caffeinate(&l);
        assert!(!l.v17);
        assert_eq!(l.argv.last().unwrap(), &s(&root.join(format!("game/{TITLE_ID}/EBOOT.elf"))));
        // the update downloaded, its menus not built: the window asks for the build
        touch(online_update_eboot(&st));
        assert!(matches!(prepare_online_launch(&st, &input, false), Err(LaunchError::OnlineBuild)));
        // built: the v1.7 menus
        touch(root.join(format!("build/{TITLE_ID}_eboot_v17/{GAME_EXE}")));
        touch(root.join(format!("game/{TITLE_ID}/EBOOT_v17.elf")));
        let l = prepare_online_launch(&st, &input, false).unwrap();
        check(&l);
        no_caffeinate(&l);
        assert!(l.v17);
        assert_eq!(l.argv.last().unwrap(), &s(&root.join(format!("game/{TITLE_ID}/EBOOT_v17.elf"))));
        assert!(l.argv.contains(&s(&root.join(format!("build/{TITLE_ID}_eboot_v17/{GAME_EXE}")))));
        // the window's own start may put caffeinate in front, nothing else changes
        let w = prepare_online_launch(&st, &input, true).unwrap();
        check(&w);
        assert_eq!(w.argv[0] == "/usr/bin/caffeinate", Path::new("/usr/bin/caffeinate").is_file());
        assert_eq!(w.argv[w.argv.len() - 3..], l.argv[l.argv.len() - 3..]);
        let _ = fs::remove_dir_all(&t);
    }

    #[test]
    fn online_removes_inherited_frame_rate_variables() {
        let v = online_inherited_removals(["HOME".to_string(), "PS3_NET_DNS".into(), "PS3_NP_TICKET".into(), "RES_UNLOCK_FPS".into(), "PS3_AA".into()]);
        for k in ["RES_UNLOCK_FPS", "RES_MENU_60FPS", "PS3_VBLANK_HZ", "PS3_FLIP_MAX_HZ", "RES_GAME_VERSION", "PS3_NET_DNS", "PS3_NP_TICKET", "PS3_MIC"] {
            assert!(v.iter().any(|x| x == k), "{k}");
        }
        assert!(!v.iter().any(|x| x == "HOME" || x == "PS3_AA"));
        assert_eq!(v.iter().filter(|x| *x == "RES_UNLOCK_FPS").count(), 1);
    }

    #[test]
    fn file_sha256_hashes_a_file() {
        let d = std::env::temp_dir().join(format!("res-sha-{}", std::process::id()));
        fs::write(&d, b"abc").unwrap();
        assert_eq!(file_sha256(&d).as_deref(), Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
        let _ = fs::remove_file(&d);
        assert_eq!(file_sha256(Path::new("/nonexistent/x")), None);
        assert_eq!(ONLINE_EBOOT_SHA.len(), 64);
    }

    #[test]
    fn online_menus_script_runs_the_eboot17_step() {
        let sc = online_menus_script(false);
        assert!(sc.contains("tools/rebuild.sh toolkit\n"));
        assert!(sc.ends_with("PY=$PWD/.venv/bin/python exec tools/rebuild.sh eboot17\n"), "{sc}");
        assert!(!sc.contains("==> done"));
    }

    #[test]
    fn launch_env_maps_the_settings() {
        let mut st = Settings::default();
        let env = |st: &Settings| -> BTreeMap<String, String> {
            launch_env(st, Path::new("/Volumes/RESISTANCE_1"), Path::new("/p"), &InputEnv::default()).into_iter().collect()
        };
        let e = env(&st);
        assert_eq!(e["PS3_VFS_ROOT"], "/Volumes/RESISTANCE_1");
        assert_eq!(e["PS3_HDD0_ROOT"], "/p/dev_hdd0");
        assert_eq!(e["PS3_HDD1_ROOT"], "/p/dev_hdd1");
        assert_eq!(e["PS3_OVERLAY_INI"], "/p/overlay.ini");
        assert_eq!(e["PS3RECOMP_METAL_FPS"], "1");
        assert_eq!(e["PS3_VERBOSE"], "0");
        assert!(e["PS3_SHADER_CACHE"].ends_with("resistance-recomp/shader-cache"));
        for k in ["RES_GAME_VERSION", "PS3_LANG", "PS3_AA", "PS3_SHARPEN", "PS3_ANISO", "PS3_DISPLAY_GAMMA", "PS3RECOMP_ENG_VCACHE", "PS3RECOMP_ENG_VCACHE_SPLIT", "PS3RECOMP_METAL_BIND_ALL", "PS3_SHADER_ASYNC", "PS3_FRAMECLOCK_US", "PS3RECOMP_FFMPEG"] {
            assert!(!e.contains_key(k), "{k} passed by default");
        }
        // the launch is the same as before the new settings existed
        let keys: Vec<&str> = e.keys().map(String::as_str).filter(|k| !k.starts_with("PS3RECOMP_FFMPEG_")).collect();
        assert_eq!(keys, ["PS3RECOMP_METAL_FPS", "PS3_HDD0_ROOT", "PS3_HDD1_ROOT", "PS3_OVERLAY_INI", "PS3_SHADER_CACHE", "PS3_VERBOSE", "PS3_VFS_ROOT"]);
        st.campaign = "100".into();
        st.game_lang = "pt-br".into();
        st.aa = "fxaa".into();
        st.sharpen = 0.3;
        st.aniso = 8;
        st.gamma_override = true;
        st.gamma = 1.1;
        st.vertex_cache_split = false;
        st.bind_changed_only = false;
        st.shader_cache = false;
        st.shader_async = true;
        st.frame_clock_us = 4000;
        st.video = false;
        st.show_fps = false;
        st.extra_env = "PS3RECOMP_METAL_FPS=1\nX=y".into();
        let e = env(&st);
        assert_eq!(e["RES_GAME_VERSION"], "100");
        assert_eq!(e["PS3_LANG"], "pt-br");
        assert_eq!(e["PS3_AA"], "fxaa");
        assert_eq!(e["PS3_SHARPEN"], "0.30");
        assert_eq!(e["PS3_ANISO"], "8");
        assert_eq!(e["PS3_DISPLAY_GAMMA"], "1.10");
        assert_eq!(e["PS3RECOMP_ENG_VCACHE_SPLIT"], "0");
        assert_eq!(e["PS3RECOMP_METAL_BIND_ALL"], "1");
        assert!(!e.contains_key("PS3_SHADER_CACHE"));
        assert_eq!(e["PS3_SHADER_ASYNC"], "1");
        assert_eq!(e["PS3_FRAMECLOCK_US"], "4000");
        assert_eq!(e["PS3RECOMP_FFMPEG"], "0");
        assert_eq!(e["PS3RECOMP_METAL_FPS"], "1", "the extra variables win");
        assert_eq!(e["X"], "y");
        st.vertex_cache = false;
        assert_eq!(env(&st)["PS3RECOMP_ENG_VCACHE"], "0");
        st.game_lang = "x; rm".into();
        assert!(!env(&st).contains_key("PS3_LANG"));
    }

    /// Every new setting, from its default (nothing passed) to a change.
    #[test]
    fn launch_env_maps_the_new_settings() {
        let env = |st: &Settings, input: &InputEnv| -> BTreeMap<String, String> {
            launch_env(st, Path::new("/d"), Path::new("/p"), input).into_iter().collect()
        };
        let base: Vec<String> = env(&Settings::default(), &InputEnv::default()).into_keys().collect();
        let new_vars = [
            "PS3_INPUT_MAP", "PS3_INPUT_MODE", "PS3_MOUSE", "PS3_MOUSE_SENS", "PS3_MOUSE_INVERT_X", "PS3_MOUSE_INVERT_Y",
            "PS3_MOUSE_DEADZONE", "PS3_MOUSE_CAPTURE", "PS3_RUMBLE", "PS3_RUMBLE_STRENGTH", "PS3_WINDOW_MODE", "PS3_WINDOW_SIZE",
            "PS3_DISPLAY", "PS3_VSYNC", "PS3_UPSCALE", "PS3_VIBRANCE", "PS3_CONTRAST", "PS3_VIGNETTE", "PS3_GRAIN", "PS3_LUT",
            "PS3_LUT_STRENGTH", "PS3_MODS", "PS3_MODS_DIR", "PS3_MODS_DUMP", "PS3_RES_SCALE", "RES_UNLOCK_FPS", "RES_MENU_60FPS",
            "RES_FOV", "PS3_AUDIO_VOLUME", "PS3_AUDIO_MUTE", "PS3_AUDIO_DEVICE", "PS3_PAD_ADAPTIVE", "RES_MOUSE_LOOK",
        ];
        for k in new_vars {
            assert!(!base.iter().any(|b| b == k), "{k} passed by default");
        }

        let mut st = Settings::default();
        st.aa = "smaa".into();
        st.vibrance = -0.25;
        st.contrast = 1.2;
        st.vignette = 0.5;
        st.grain = 0.1;
        st.lut_path = "/luts/warm.png".into();
        st.lut_strength = 0.6;
        st.res_scale = "1.5".into();
        st.mods = vec!["HD UI".into(), "a,b".into()];
        st.mods_dump = true;
        st.window_mode = "borderless".into();
        st.window_size = "1920x1080".into();
        st.display = "DELL U2720Q".into();
        st.vsync = false;
        st.upscale = "metalfx".into();
        st.input_mode = "keyboard".into();
        st.mouse_sens = 1.5;
        st.mouse_invert_y = true;
        st.mouse_deadzone = 10;
        st.mouse_capture = false;
        st.rumble_strength = 150;
        st.unlock_fps = true;
        st.fov = "0.7".into();
        st.audio_volume = 40;
        st.audio_device = "AirPods".into();
        let input = InputEnv { path: Some("/data/input.toml".into()), has_mouse_section: false };
        let e = env(&st, &input);
        assert_eq!(e["PS3_AA"], "smaa");
        assert_eq!(e["PS3_VIBRANCE"], "-0.25");
        assert_eq!(e["PS3_CONTRAST"], "1.20");
        assert_eq!(e["PS3_VIGNETTE"], "0.50");
        assert_eq!(e["PS3_GRAIN"], "0.10");
        assert_eq!(e["PS3_LUT"], "/luts/warm.png");
        assert_eq!(e["PS3_LUT_STRENGTH"], "0.60");
        assert_eq!(e["PS3_RES_SCALE"], "1.5");
        assert_eq!(e["PS3_MODS"], "HD UI");
        assert!(e["PS3_MODS_DIR"].ends_with("resistance-recomp/mods"));
        assert!(e["PS3_MODS_DUMP"].ends_with("resistance-recomp/texture-dump"));
        assert_eq!(e["PS3_WINDOW_MODE"], "borderless");
        assert!(!e.contains_key("PS3_WINDOW_SIZE"), "the size stays the host's --size");
        assert_eq!(e["PS3_DISPLAY"], "DELL U2720Q");
        assert_eq!(e["PS3_VSYNC"], "0");
        assert_eq!(e["PS3_UPSCALE"], "metalfx");
        assert_eq!(e["PS3_INPUT_MAP"], "/data/input.toml");
        assert_eq!(e["PS3_INPUT_MODE"], "keyboard");
        assert!(!e.contains_key("PS3_MOUSE"));
        assert_eq!(e["PS3_MOUSE_SENS"], "1.50");
        assert_eq!(e["PS3_MOUSE_INVERT_Y"], "1");
        assert!(!e.contains_key("PS3_MOUSE_INVERT_X"));
        assert_eq!(e["PS3_MOUSE_DEADZONE"], "10");
        assert_eq!(e["PS3_MOUSE_CAPTURE"], "0");
        assert!(!e.contains_key("PS3_RUMBLE"));
        assert_eq!(e["PS3_RUMBLE_STRENGTH"], "150");
        assert_eq!(e["RES_UNLOCK_FPS"], "1");
        assert_eq!(e["RES_MENU_60FPS"], "1");
        assert_eq!(e["RES_FOV"], "0.70");
        assert_eq!(e["PS3_AUDIO_VOLUME"], "40");
        assert_eq!(e["PS3_AUDIO_DEVICE"], "AirPods");
        assert!(!e.contains_key("PS3_AUDIO_MUTE"));
        // muted: the mute alone (any value mutes); a saved menus-only switch
        // still means 60 fps everywhere
        st.audio_mute = true;
        st.audio_device = " a\nb ".into();
        st.unlock_fps = false;
        st.menu_60fps = true;
        let e = env(&st, &input);
        assert_eq!(e["PS3_AUDIO_MUTE"], "1");
        assert!(!e.contains_key("PS3_AUDIO_VOLUME"));
        assert!(!e.contains_key("PS3_AUDIO_DEVICE"));
        assert_eq!(e["RES_UNLOCK_FPS"], "1");
        assert_eq!(e["RES_MENU_60FPS"], "1");

        // a hand-written [mouse] in the file: every mouse setting is passed
        let mut st = Settings::default();
        let e = env(&st, &InputEnv { path: Some("/x.toml".into()), has_mouse_section: true });
        assert_eq!(e["PS3_MOUSE"], "1");
        assert_eq!(e["PS3_MOUSE_SENS"], "1.00");
        assert_eq!(e["PS3_MOUSE_INVERT_X"], "0");
        assert_eq!(e["PS3_MOUSE_INVERT_Y"], "0");
        assert_eq!(e["PS3_MOUSE_DEADZONE"], "30");
        assert_eq!(e["PS3_MOUSE_CAPTURE"], "1");
        assert!(!e.contains_key("RES_MOUSE_LOOK")); // experimental: off unless asked for
        st.mouse_direct = true;
        assert_eq!(env(&st, &InputEnv::default())["RES_MOUSE_LOOK"], "1");
        // mouse and rumble off; values out of range or unknown pass nothing
        st.mouse = false;
        st.rumble = false;
        st.adaptive_triggers = false;
        st.window_mode = "windowed".into();
        st.input_mode = "both".into();
        st.aa = "msaa".into();
        st.res_scale = "1".into();
        st.fov = "1.0".into();
        st.display = "a\nb".into();
        st.lut_strength = 0.5; // no LUT file: no strength either
        let e = env(&st, &InputEnv::default());
        assert_eq!(e["PS3_MOUSE"], "0");
        assert!(!e.contains_key("PS3_MOUSE_SENS"));
        assert!(!e.contains_key("RES_MOUSE_LOOK")); // no mouse, no mouse look
        assert_eq!(e["PS3_RUMBLE"], "0");
        assert_eq!(e["PS3_PAD_ADAPTIVE"], "0");
        for k in ["PS3_WINDOW_MODE", "PS3_INPUT_MODE", "PS3_AA", "PS3_RES_SCALE", "RES_FOV", "PS3_DISPLAY", "PS3_LUT_STRENGTH", "PS3_INPUT_MAP"] {
            assert!(!e.contains_key(k), "{k}");
        }
    }

    #[test]
    fn res_scale_and_fov_values() {
        assert_eq!(res_scale_value(""), None);
        assert_eq!(res_scale_value("1"), None);
        assert_eq!(res_scale_value("1.5").as_deref(), Some("1.5"));
        assert_eq!(res_scale_value("2").as_deref(), Some("2"));
        assert_eq!(res_scale_value("3").as_deref(), Some("3"));
        assert_eq!(res_scale_value("2.6").as_deref(), Some("2.5"));
        assert_eq!(res_scale_value("5"), None);
        assert_eq!(res_scale_value("1080p"), None);
        assert_eq!(fov_value("0.7").as_deref(), Some("0.70"));
        assert_eq!(fov_value("1"), None);
        assert_eq!(fov_value("9"), None);
        assert_eq!(fov_value("wide"), None);
    }

    /// A launcher.json from before these settings (the installed app's)
    /// loads with the new fields at their defaults, and keeps its own.
    #[test]
    fn old_settings_files_still_load() {
        let old = r#"{"portDir": "/Users/x/resistence", "discPath": "/Volumes/RESISTANCE_1", "playDir": "/p", "campaign": "auto", "aa": "fxaa"}"#;
        let st: Settings = serde_json::from_str(old).unwrap();
        assert_eq!(st.port_dir, "/Users/x/resistence");
        assert_eq!(st.aa, "fxaa");
        assert_eq!(st.window_mode, "windowed");
        assert!(st.vsync && st.mouse && st.rumble && st.mouse_capture);
        assert_eq!((st.mouse_sens, st.mouse_deadzone, st.rumble_strength), (1.0, 30, 100));
        assert_eq!(st.input_mode, "both");
        assert!(st.mods.is_empty() && !st.unlock_fps && st.fov.is_empty());
        let e: BTreeMap<String, String> = launch_env(&st, Path::new("/d"), Path::new("/p"), &InputEnv::default()).into_iter().collect();
        let keys: Vec<&str> = e.keys().map(String::as_str).filter(|k| !k.starts_with("PS3RECOMP_FFMPEG_")).collect();
        assert_eq!(keys, ["PS3RECOMP_METAL_FPS", "PS3_AA", "PS3_HDD0_ROOT", "PS3_HDD1_ROOT", "PS3_OVERLAY_INI", "PS3_SHADER_CACHE", "PS3_VERBOSE", "PS3_VFS_ROOT"]);
    }

    #[test]
    fn input_map_file_is_written_and_kept() {
        let d = tmp("input");
        let p = d.join("sub/input.toml");
        let mut b = Bindings::default();
        b.keyboard.insert("r1".into(), vec!["X".into()]);
        b.keyboard.insert("cross".into(), vec!["Space".into()]);
        b.keyboard.insert("l3".into(), vec![]);
        b.mouse_buttons.insert("r1".into(), vec!["Left".into()]);
        b.mouse_buttons.insert("triangle".into(), vec!["WheelUp".into(), "WheelDown".into()]);
        b.controller.insert("cross".into(), vec!["a".into()]);
        write_input_map(&p, b.clone()).unwrap();
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.starts_with("# Resistance recomp: controls"));
        assert!(text.contains("[keyboard]\ncross    = \"Space\"\n"));
        assert!(text.contains("l3       = \"\"\n"));
        assert!(text.contains("[mouse_buttons]\ntriangle = \"WheelUp, WheelDown\"\nr1       = \"Left\"\n"));
        assert!(text.contains("[controller]\ncross    = \"a\"\n"));
        assert_eq!(input_map::parse(&text).bindings, b);
        // a hand-written [mouse] survives the next save
        fs::write(&p, format!("{text}\n[mouse]\nsensitivity = 2.0\n")).unwrap();
        b.keyboard.insert("r1".into(), vec!["F".into()]);
        write_input_map(&p, b.clone()).unwrap();
        let f = input_map::parse(&fs::read_to_string(&p).unwrap());
        assert_eq!(f.bindings.keyboard["r1"], vec!["F"]);
        assert!(f.has_mouse_section && f.other.contains("sensitivity = 2.0"));
        // names that could break the file are refused, and the file stays
        b.keyboard.insert("cross".into(), vec!["Q\"\n[x]".into()]);
        assert!(write_input_map(&p, b).is_err());
        assert_eq!(input_map::parse(&fs::read_to_string(&p).unwrap()).bindings.keyboard["cross"], vec!["Space"]);
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn feature_markers_in_a_build() {
        let d = tmp("features");
        let exe = d.join("game");
        fs::write(&exe, b"\x7fELF junk PS3_INPUT_MAP\0more RES_FOV=%s\0 RES_FOV\0 PS3_RES_SCALE_X\0").unwrap();
        let b = Builds { eboot: Some(s(&exe)), ..Default::default() };
        let cache = Mutex::new(FeatureCache::new());
        assert_eq!(build_features(&b, &cache), vec!["PS3_INPUT_MAP".to_string(), "RES_FOV".to_string()]);
        assert_eq!(cache.lock().unwrap().len(), 1);
        assert_eq!(build_features(&b, &cache).len(), 2, "from the cache");
        assert!(build_features(&Builds::default(), &cache).is_empty());
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn screen_names() {
        assert_eq!(parse_screens("[\"Built-in Retina Display\",\"DELL U2720Q\"]\n"), vec!["Built-in Retina Display", "DELL U2720Q"]);
        assert!(parse_screens("oops").is_empty());
    }

    #[test]
    fn windows_drive_roots_and_quoting() {
        assert_eq!(drive_letter(Path::new("e:\\")), Some('E'));
        assert_eq!(drive_letter(Path::new("F:")), Some('F'));
        assert_eq!(drive_letter(Path::new("F:/")), Some('F'));
        assert_eq!(drive_letter(Path::new("F:\\PS3_GAME")), None);
        assert_eq!(drive_letter(Path::new("/Volumes/X")), None);
    }

    #[test]
    fn powershell_inputs_go_through_the_environment() {
        // U+2019 is a single quote to PowerShell: a path must never be source
        let image = "C:\\x\u{2019}; calc; \u{2019}'; calc; '.iso";
        let cmd = powershell_cmd(PS_ATTACH, &[("RES_IMAGE", std::ffi::OsStr::new(image))]);
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args.last(), Some(&ps_encoded(PS_ATTACH)));
        assert_eq!(ps_encoded("ab"), "YQBiAA==");
        for a in &args {
            assert!(!a.contains("calc") && !a.contains('\u{2019}'), "{a}");
        }
        let env: Vec<_> = cmd.get_envs().collect();
        assert_eq!(env, vec![(std::ffi::OsStr::new("RES_IMAGE"), Some(std::ffi::OsStr::new(image)))]);
        assert!(PS_ATTACH.contains("$env:RES_IMAGE") && PS_DETACH.contains("$env:RES_DRIVE"));
    }

    #[test]
    fn window_sizes() {
        assert!(valid_window_size("1280x720"));
        assert!(valid_window_size("2560x1440"));
        assert!(!valid_window_size(""));
        assert!(!valid_window_size("10x10"));
        assert!(!valid_window_size("1280x720x1"));
    }

    #[test]
    fn rebuild_script_steps() {
        let all = rebuild_script(&[], false);
        assert!(all.starts_with("set -e\ntools/rebuild.sh toolkit\n"));
        assert!(all.contains("exec tools/rebuild.sh decrypt analyse lift spu build\n"));
        assert!(all.contains("-m venv .venv"));
        let some = rebuild_script(&["build".into()], false);
        assert!(some.contains("exec tools/rebuild.sh build\n"));
        let tk = rebuild_script(&["toolkit".into()], false);
        assert!(tk.contains("tools/rebuild.sh toolkit\n") && tk.contains("==> done"));
        let deps = rebuild_script(&["build".into()], true);
        assert!(deps.starts_with("set -e\necho '==> deps'\ntools/bootstrap_deps.sh\ntools/rebuild.sh toolkit\n"));
    }

    #[test]
    fn install_script_clones_or_reuses() {
        let t = tmp("install-script");
        // no checkout: clone into src at the tag, then tools/install.sh
        let src = t.join("Application Support/src");
        let sc = install_script(None, &src, "https://example.org/r.git", "v1.2.3");
        assert!(sc.contains("==> sources"));
        assert!(sc.contains(&format!("SRC='{}'\n", src.display())));
        assert!(sc.contains("git clone --quiet https://example.org/r.git \"$SRC.partial\""));
        assert!(sc.contains("checkout --quiet -B \"$br\" v1.2.3"));
        assert!(sc.ends_with("cd \"$SRC\"\nexec tools/install.sh\n"));
        // a checkout with tools/install.sh
        let repo = t.join("repo");
        fs::create_dir_all(repo.join("tools")).unwrap();
        fs::write(repo.join("tools/rebuild.sh"), "").unwrap();
        let old = install_script(Some(&repo), &src, "u", "v1");
        assert!(!old.contains("==> sources") && old.contains("exec tools/rebuild.sh decrypt analyse lift spu build"));
        assert!(!old.contains("bootstrap_deps"));
        fs::write(repo.join("tools/bootstrap_deps.sh"), "").unwrap();
        assert!(install_script(Some(&repo), &src, "u", "v1").contains("tools/bootstrap_deps.sh\n"));
        fs::write(repo.join("tools/install.sh"), "").unwrap();
        let new = install_script(Some(&repo), &src, "u", "v1");
        assert_eq!(new, format!("set -e\ncd {}\nexec tools/install.sh\n", shell_quote(&s(&repo))));
        let _ = fs::remove_dir_all(&t);
    }

    #[test]
    fn deps_ready_needs_the_manifest_and_ffmpeg() {
        let t = tmp("deps-ready");
        assert!(!deps_ready(&t));
        fs::create_dir_all(t.join("include/libavcodec")).unwrap();
        fs::write(t.join("include/libavcodec/avcodec.h"), "").unwrap();
        assert!(!deps_ready(&t));
        fs::write(t.join("manifest.txt"), "cmake 4.4.4\n").unwrap();
        assert!(deps_ready(&t));
        let _ = fs::remove_dir_all(&t);
    }
}
