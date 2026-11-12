//! RecompLauncher (recomplauncher.com, a third-party launcher that installs
//! recomps from their GitHub releases) starts this launcher two ways, both
//! decided in `run()` before Tauri, so no window, event loop or updater runs:
//!
//! - `--prepare --game <disc>`: its setup step, no window. What Home's
//!   "Install the game" does for a player who has nothing yet (or an install
//!   to bring up to date): the disc checked and remembered, Apple's Command
//!   Line Tools, the port's sources (cloned, or the install's own checkout
//!   updated), then tools/install.sh (the build tools, the toolkit, decrypt,
//!   lift, build). `progress <0..1> <text>` lines on stdout as it goes, the
//!   install's output as plain lines, and an exit code from `Exit`.
//! - `--play [--game <disc>]` with RECOMPLAUNCHER=1 in the environment (it
//!   sets that for every game it starts): the window's Play without the
//!   window. The launcher replaces itself with the game (exec), so the
//!   process RecompLauncher waits for is the game and its exit code is the
//!   game's; a check that fails first exits with an `Exit` code.
//! - `--play-online [--game <disc>]` with RECOMPLAUNCHER=1 (the profile's
//!   second tile, "Online (PS Rewired)", adds it after `--play`): Home's Play
//!   online without the window, the same checks and online environment
//!   (never a frame-rate setting). When the game has downloaded its v1.7
//!   update and the v1.7 menus are not built yet, they are built first
//!   (`progress` lines); before the update is there, the disc's menus run so
//!   that Multiplayer > Online Game downloads it.
//!
//! `--prepare` also builds the v1.7 menus after the campaign when the update
//! is already in the online folder; that part failing still exits 0 (the
//! campaign is ready), with a line saying so, and `--play-online` tries again.
//!
//! The profile that tells RecompLauncher all this is recomplauncher.json at
//! the repository's root; launcher/scripts/package-recomplauncher.sh puts it
//! beside the .app in the release zip.
//!
//! Without RECOMPLAUNCHER=1, `--play` stays the Play shortcut's (the window
//! opens and starts the game) and `--play-online` opens the window and
//! starts online play as its button; `--game <disc>` is remembered first.

use super::*;
use std::io::Write;

pub const USAGE: &str =
    "usage: resistance-launcher [--prepare --game <disc .iso or folder>] [--play [--offline | --play-online] [--game <disc>]] [--play-online [--game <disc>]]";

/// Online play (PS Rewired) instead of the offline game.
pub const ONLINE_ARG: &str = "--play-online";
/// The offline game, said explicitly (the profile's first tile).
pub const OFFLINE_ARG: &str = "--offline";

/// The headless modes' exit codes (recomplauncher.json `exit_codes` has the
/// player's text for each; launcher/README.md lists them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Ok = 0,
    /// A malformed command line (`--game` without a path, `--prepare` with `--play`).
    Usage = 2,
    /// No disc given, the path is not there, or it is not the US disc BCUS98107.
    GameCopy = 10,
    /// Apple's Command Line Tools are missing: their installer was opened, the
    /// player finishes it and runs the setup again.
    NeedsTools = 11,
    /// The port's sources (or the build's downloads) could not be fetched.
    Network = 12,
    /// The install stopped in a build step, or ended without the game built.
    Build = 13,
    /// Another install, or the game, is running.
    Running = 14,
    /// Not enough free disk space for the install.
    Disk = 15,
    /// Not a Mac: the install and the game are macOS only.
    Unsupported = 16,
    /// `--play` before the game is installed (or with an incomplete build).
    NotInstalled = 17,
    /// `--play`: the v1.7 update's files are in the play folder; the launcher
    /// window moves them aside (it asks first).
    UpdateFiles = 18,
    /// `--play-online` without a valid online ID: it is typed once in the
    /// launcher window (which is opened for it).
    OnlineId = 19,
    /// `--play-online`: the v1.7 update is downloaded, and building the v1.7
    /// menus from it failed (or it is not the tested EBOOT.BIN).
    OnlineBuild = 20,
}

impl Exit {
    #[cfg(test)]
    pub const ALL: [Exit; 13] = [
        Exit::Ok,
        Exit::Usage,
        Exit::GameCopy,
        Exit::NeedsTools,
        Exit::Network,
        Exit::Build,
        Exit::Running,
        Exit::Disk,
        Exit::Unsupported,
        Exit::NotInstalled,
        Exit::UpdateFiles,
        Exit::OnlineId,
        Exit::OnlineBuild,
    ];

    pub fn code(self) -> i32 {
        self as i32
    }
}

/// A failed check: the exit code and what to tell the player.
#[derive(Debug)]
pub struct Fail {
    pub exit: Exit,
    msg: String,
}

impl Fail {
    fn new(exit: Exit, msg: impl Into<String>) -> Self {
        Fail { exit, msg: msg.into() }
    }

    pub fn message(&self) -> &str {
        &self.msg
    }
}

// ---------------------------------------------------------------------------
// The command line
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum Mode {
    /// The window, as always (`--play` there is the Play shortcut's); `game`:
    /// a disc to use and remember first; `online`: start online play once
    /// the window is up (`--play-online`).
    Window { game: Option<PathBuf>, online: bool },
    Prepare { game: Option<PathBuf> },
    PlayHeadless { game: Option<PathBuf> },
    PlayOnlineHeadless { game: Option<PathBuf> },
}

/// What the command line (argv, program first) asks for. `recomplauncher`:
/// RECOMPLAUNCHER=1 is set (only then are `--play` and `--play-online`
/// headless). `--play-online` alone, or after `--play`, is online play.
/// Arguments it does not know are left to the window (macOS's own
/// `-psn_...`, `--view`).
pub fn mode(args: &[String], recomplauncher: bool) -> Result<Mode, String> {
    let (mut prepare, mut play, mut online, mut offline, mut game) = (false, false, false, false, None::<PathBuf>);
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        if a == "--prepare" {
            prepare = true;
        } else if a == play_shortcut::ARG {
            play = true;
        } else if a == ONLINE_ARG {
            online = true;
        } else if a == OFFLINE_ARG {
            offline = true;
        } else if let Some(v) = a.strip_prefix("--game=") {
            game = Some(PathBuf::from(v));
        } else if a == "--game" {
            match it.next() {
                Some(v) if !v.starts_with("--") => game = Some(PathBuf::from(v)),
                _ => return Err("--game needs the path of your disc (.iso or folder)".into()),
            }
        }
    }
    // an empty path (a blank {game}) is no path
    let game = game.filter(|g| !g.as_os_str().is_empty());
    if online && offline {
        return Err(format!("{ONLINE_ARG} and {OFFLINE_ARG} are two different games"));
    }
    match (prepare, play || online) {
        (true, true) => Err("--prepare and --play are two separate runs".into()),
        (true, false) => Ok(Mode::Prepare { game }),
        (false, true) if recomplauncher && online => Ok(Mode::PlayOnlineHeadless { game }),
        (false, true) if recomplauncher => Ok(Mode::PlayHeadless { game }),
        _ => Ok(Mode::Window { game, online }),
    }
}

// ---------------------------------------------------------------------------
// The player's copy
// ---------------------------------------------------------------------------

/// A checked copy of BCUS98107.
#[derive(Debug, PartialEq)]
pub enum Copy {
    /// The folder that holds PS3_GAME/ (a mounted disc or an extracted copy).
    Folder(PathBuf),
    /// A disc image (.iso).
    Image(PathBuf),
}

/// Read one file out of an ISO 9660 image by its path (`["PS3_GAME",
/// "PARAM.SFO"]`, matched without case or `;1`), from the primary volume
/// descriptor's directory tree. None when the image is not ISO 9660 or the
/// file is not there (or over `max` bytes).
pub(crate) fn iso_read(path: &Path, parts: &[&str], max: usize) -> Option<Vec<u8>> {
    let mut f = fs::File::open(path).ok()?;
    let mut at = |off: u64, len: usize| -> Option<Vec<u8>> {
        f.seek(SeekFrom::Start(off)).ok()?;
        let mut b = vec![0u8; len];
        f.read_exact(&mut b).ok()?;
        Some(b)
    };
    let pvd = at(16 * 2048, 2048)?;
    if pvd[0] != 1 || &pvd[1..6] != b"CD001" {
        return None;
    }
    let le = |b: &[u8], i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as u64;
    let root = &pvd[156..190];
    let (mut lba, mut size) = (le(root, 2), le(root, 10));
    for (n, part) in parts.iter().enumerate() {
        let last = n + 1 == parts.len();
        let dir = at(lba * 2048, (size as usize).min(4 << 20))?;
        let want = part.to_ascii_uppercase();
        let mut i = 0;
        let mut found = None;
        while i < dir.len() {
            let len = dir[i] as usize;
            if len == 0 {
                // records never cross a sector: the rest of this one is padding
                i = (i / 2048 + 1) * 2048;
                continue;
            }
            if len < 34 || i + len > dir.len() {
                break;
            }
            let r = &dir[i..i + len];
            let nl = r[32] as usize;
            if 33 + nl <= len {
                let name = String::from_utf8_lossy(&r[33..33 + nl]).to_ascii_uppercase();
                let is_dir = r[25] & 2 != 0;
                if name.split(';').next() == Some(want.as_str()) && is_dir != last {
                    found = Some((le(r, 2), le(r, 10)));
                    break;
                }
            }
            i += len;
        }
        (lba, size) = found?;
    }
    if size as usize > max {
        return None;
    }
    at(lba * 2048, size as usize)
}

/// Whether `path` is the US disc BCUS98107: a folder holding PS3_GAME/ (or
/// PS3_GAME itself), or an .iso, read without mounting it. Read only.
pub fn check_copy(path: &Path) -> Result<Copy, Fail> {
    let bad = |m: String| Fail::new(Exit::GameCopy, m);
    if path.is_dir() {
        let root = disc_root_of(path).ok_or_else(|| bad(format!("{}: no PS3_GAME/PARAM.SFO in it (not a PS3 disc folder)", path.display())))?;
        return match disc_title_id(&root) {
            Some(id) if id == TITLE_ID => Ok(Copy::Folder(root)),
            Some(id) => Err(bad(format!("{}: that is {id}, another game or region; this needs the US disc {TITLE_ID}", path.display()))),
            None => Err(bad(format!("{}: its PARAM.SFO cannot be read", path.display()))),
        };
    }
    if !path.is_file() {
        return Err(bad(format!("{}: no such file or folder", path.display())));
    }
    let sfo = iso_read(path, &["PS3_GAME", "PARAM.SFO"], 1 << 20)
        .ok_or_else(|| bad(format!("{}: not a PS3 disc image this can read (a plain, uncompressed .iso of the disc, or its folder)", path.display())))?;
    match saves::sfo_values(&sfo).get("TITLE_ID") {
        Some(id) if id == TITLE_ID => Ok(Copy::Image(path.to_path_buf())),
        Some(id) => Err(bad(format!("{}: that is {id}, another game or region; this needs the US disc {TITLE_ID}", path.display()))),
        None => Err(bad(format!("{}: its PARAM.SFO cannot be read", path.display()))),
    }
}

/// Check the copy, then make it the launcher's disc, as the window's "Pick the
/// disc" does: an image is attached (read only) and remembered with its mount
/// point; a folder replaces any remembered image. Returns the disc's root.
pub fn use_disc(path: &Path) -> Result<PathBuf, Fail> {
    let copy = check_copy(path)?;
    let mut st = load_settings_file();
    let root = match copy {
        Copy::Folder(root) => {
            st.disc_image.clear();
            root
        }
        Copy::Image(img) => {
            let root = attach_image(&img).map_err(|e| Fail::new(Exit::GameCopy, format!("could not mount {}: {e}", img.display())))?;
            st.disc_image = s(&img);
            root
        }
    };
    st.disc_path = s(&root);
    write_settings(&st).map_err(|e| Fail::new(Exit::Build, format!("could not save the settings: {e}")))?;
    Ok(root)
}

// ---------------------------------------------------------------------------
// Progress
// ---------------------------------------------------------------------------

/// The install's steps as tools/install.sh announces them ("==> <step>"),
/// with their rough share of a full install (as the window's progress.ts) and
/// what the player is told.
const STEPS: &[(&str, f64, &str)] = &[
    ("sources", 1.0, "Downloading the port's sources"),
    ("deps", 8.0, "Installing the build tools and libraries"),
    ("toolkit", 2.0, "Setting up the recompiler"),
    ("venv", 3.0, "Making the Python environment"),
    ("decrypt", 2.0, "Decrypting the game from your disc"),
    ("analyse", 6.0, "Analysing the game's code"),
    ("lift", 35.0, "Recompiling the game's code"),
    ("spu", 6.0, "Recompiling the SPU programs"),
    ("build", 37.0, "Compiling the game"),
];

/// The v1.7 menus' build (tools/rebuild.sh toolkit, then eboot17, whose
/// tools/v17/regen_eboot_v17.sh says "==> v1.7 menus: <step>"), as
/// online_menus_script runs it. One build target.
const ONLINE_STEPS: &[(&str, f64, &str)] = &[
    ("deps", 8.0, "Installing the build tools and libraries"),
    ("toolkit", 2.0, "Setting up the recompiler"),
    ("venv", 2.0, "Making the Python environment"),
    ("decrypt", 1.0, "Decrypting the online menus from the game's v1.7 update"),
    ("analyse", 6.0, "Analysing the online menus' code"),
    ("lift", 40.0, "Recompiling the online menus"),
    ("spu", 6.0, "Recompiling the online menus' SPU programs"),
    ("build", 35.0, "Compiling the online menus"),
];

/// Where the install's share starts and how much of 0..1 it takes: before it
/// come the copy, the tools and the sources' update; 1.0 is "ready".
const INSTALL_FROM: f64 = 0.03;
const INSTALL_SPAN: f64 = 0.96;
/// With the v1.7 menus to build after the campaign (the update is already
/// in the online folder): the install takes up to here, they take the rest.
const ONLINE_FROM: f64 = 0.80;

fn strip_esc(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut it = line.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\x1b' && it.peek() == Some(&'[') {
            it.next();
            for d in it.by_ref() {
                if d.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// "[k/n]" at the start of `s` (after `==> ` for tools/bootstrap_deps.sh,
/// bare for ninja).
fn counter(s: &str) -> Option<(u64, u64)> {
    let r = s.strip_prefix('[')?;
    let (k, rest) = r.split_once('/')?;
    let (n, _) = rest.split_once(']')?;
    let (k, n) = (k.trim().parse().ok()?, n.trim().parse().ok()?);
    (n > 0 && k <= n).then_some((k, n))
}

/// The install's progress, read from its output: the step from "==> <step>",
/// inside it "==> [k/n]" (the build tools) and ninja's "[k/n]" (the build's
/// two or more targets), and otherwise time (half the step after two minutes,
/// never past 90% of it) as the window does. Default: the install's steps;
/// Tracker::online: the v1.7 menus' build.
pub(crate) struct Tracker {
    steps: &'static [(&'static str, f64, &'static str)],
    /// The builds ninja runs in the build step, one after the other.
    targets: u32,
    /// What it says before the first step.
    idle: &'static str,
    cur: Option<usize>,
    /// The step's "==> done" (tools/rebuild.sh ends each run with one).
    cur_done: bool,
    sub: f64,
    since: f64,
    ninja_runs: u32,
    ninja_last: u64,
}

impl Default for Tracker {
    fn default() -> Self {
        Tracker::new(STEPS, 2, "Starting the install")
    }
}

impl Tracker {
    fn new(steps: &'static [(&'static str, f64, &'static str)], targets: u32, idle: &'static str) -> Self {
        Tracker { steps, targets, idle, cur: None, cur_done: false, sub: 0.0, since: 0.0, ninja_runs: 0, ninja_last: 0 }
    }

    pub fn online() -> Self {
        Tracker::new(ONLINE_STEPS, 1, "Starting the online menus' build")
    }

    /// Read one output line (at `now` seconds); true when a step started.
    pub fn feed(&mut self, raw: &str, now: f64) -> bool {
        let line = strip_esc(raw);
        let line = line.trim_end();
        if let Some(rest) = line.strip_prefix("==>") {
            let rest = rest.trim_start();
            // tools/v17/regen_eboot_v17.sh's steps
            let rest = rest.strip_prefix("v1.7 menus:").map(str::trim_start).unwrap_or(rest);
            if let Some((k, n)) = counter(rest) {
                self.sub = self.sub.max((k.saturating_sub(1)) as f64 / n as f64);
                return false;
            }
            let word: String = rest.chars().take_while(|c| c.is_ascii_lowercase()).collect();
            if word == "done" {
                self.cur_done = true;
                return false;
            }
            if let Some(i) = self.steps.iter().position(|(id, _, _)| *id == word) {
                // never back to an earlier step: the bar only moves on
                if self.cur.is_none_or(|c| i > c) {
                    *self = Tracker { cur: Some(i), since: now, ..Tracker::new(self.steps, self.targets, self.idle) };
                    return true;
                }
            }
            return false;
        }
        if self.step() == Some("build") {
            if let Some((k, n)) = counter(line) {
                if k < self.ninja_last {
                    self.ninja_runs += 1;
                }
                self.ninja_last = k;
                // the install: two targets as a rule (the menus, the campaign)
                self.sub = self.sub.max(((self.ninja_runs as f64 + k as f64 / n as f64) / self.targets.max(1) as f64).min(0.98));
            }
        }
        false
    }

    pub fn step(&self) -> Option<&'static str> {
        self.cur.map(|i| self.steps[i].0)
    }

    pub fn text(&self) -> &'static str {
        self.cur.map_or(self.idle, |i| self.steps[i].2)
    }

    /// The run's share done, 0..1.
    pub fn fraction(&self, now: f64) -> f64 {
        let total: f64 = self.steps.iter().map(|s| s.1).sum();
        let Some(i) = self.cur else { return 0.0 };
        let before: f64 = self.steps[..i].iter().map(|s| s.1).sum();
        let part = if self.cur_done {
            1.0
        } else {
            let el = (now - self.since).max(0.0);
            self.sub.max((el / (el + 120.0)).min(0.9))
        };
        ((before + self.steps[i].1 * part) / total).clamp(0.0, 1.0)
    }
}

/// One `progress` line: the fraction (0..1, three decimals) and a short text.
pub(crate) fn progress_line(f: f64, text: &str) -> String {
    let f = if f.is_finite() { f.clamp(0.0, 1.0) } else { 0.0 };
    format!("progress {f:.3} {}", text.replace(['\r', '\n'], " ").trim())
}

/// A plain output line, never taken for a `progress` one.
fn log_line(raw: &str) -> String {
    let l = strip_esc(raw);
    if l == "progress" || l.starts_with("progress ") {
        format!("  {l}")
    } else {
        l
    }
}

/// The prepare's standard output: progress lines that only ever go up, plain
/// lines otherwise. A closed stdout is not an error (the install goes on).
#[derive(Default)]
pub(crate) struct Out {
    last: Mutex<f64>,
}

impl Out {
    fn write(text: &str) {
        let mut o = std::io::stdout().lock();
        let _ = writeln!(o, "{text}");
        let _ = o.flush();
    }

    /// Print `progress` at `f` (or where the bar already is, if that is
    /// further); returns the fraction printed.
    pub fn progress(&self, f: f64, text: &str) -> f64 {
        let mut last = self.last.lock().unwrap();
        let f = f.clamp(0.0, 1.0).max(*last);
        *last = f;
        Self::write(&progress_line(f, text));
        f
    }

    pub fn at(&self) -> f64 {
        *self.last.lock().unwrap()
    }

    pub fn line(&self, text: &str) {
        Self::write(&log_line(text));
    }
}

// ---------------------------------------------------------------------------
// --prepare
// ---------------------------------------------------------------------------

/// What a failed install's output says went wrong (as the window's
/// installError): the disk, the network, a running install, the disc, else
/// the step it stopped in (the sources' step counts as the network's).
pub(crate) fn classify_failure(log: &str, step: Option<&str>) -> Exit {
    let has = |pats: &[&str]| pats.iter().any(|p| log.to_ascii_lowercase().contains(&p.to_ascii_lowercase()));
    if has(&["No space left on device", "not enough free disk space"]) {
        Exit::Disk
    } else if has(&["another install is already running"]) {
        Exit::Running
    } else if has(&["could not download", "Could not resolve host", "Failed to connect", "unable to access", "Connection timed out", "Network is unreachable", "Check the internet connection"]) {
        Exit::Network
    } else if has(&["the disc is not mounted"]) {
        Exit::GameCopy
    } else if step.is_none_or(|s| s == "sources") {
        Exit::Network
    } else {
        Exit::Build
    }
}

/// The last lines of a run's output, for classify_failure.
#[derive(Default)]
struct Tail(Mutex<std::collections::VecDeque<String>>);

impl Tail {
    fn push(&self, l: &str) {
        let mut v = self.0.lock().unwrap();
        if v.len() >= 400 {
            v.pop_front();
        }
        v.push_back(l.to_string());
    }

    fn text(&self) -> String {
        self.0.lock().unwrap().iter().cloned().collect::<Vec<_>>().join("\n")
    }
}

/// The game is running (started by the window, a terminal or RecompLauncher).
fn game_running() -> bool {
    process_running(GAME_EXE)
}

/// `--prepare --game <disc>`. Prints as it goes; the exit code says how it
/// ended (see Exit). Exit::Ok only when the game's builds are there after.
pub fn prepare(game: Option<&Path>) -> Exit {
    let out = Out::default();
    match prepare_with(&out, game) {
        Ok(()) => Exit::Ok,
        Err(f) => {
            out.progress(out.at(), &format!("Setup failed: {}", first_line(f.message())));
            out.line(&format!("ERROR: {}", f.message()));
            out.line(&format!("exit code {} ({:?})", f.exit.code(), f.exit));
            f.exit
        }
    }
}

fn first_line(m: &str) -> &str {
    m.lines().next().unwrap_or(m)
}

fn prepare_with(out: &Out, game: Option<&Path>) -> Result<(), Fail> {
    out.progress(0.0, "Checking your copy of the game");
    if !cfg!(target_os = "macos") {
        return Err(Fail::new(Exit::Unsupported, format!("the install and the game are macOS only; this is {}", std::env::consts::OS)));
    }
    let game = game.ok_or_else(|| Fail::new(Exit::GameCopy, format!("no disc given: --prepare --game <your {TITLE_ID} .iso or disc folder>")))?;
    // read only so far: nothing is written before the copy checks out
    let copy = check_copy(game)?;
    out.line(&format!("your copy: {} ({TITLE_ID})", game.display()));

    // one install at a time (the window's included), never under the game
    let _lock = match install_lock() {
        Ok(Some(f)) => f,
        Ok(None) => return Err(Fail::new(Exit::Running, "an install is already running (the launcher window, or another setup): wait for it to finish")),
        Err(e) => return Err(Fail::new(Exit::Build, format!("the install lock: {e}"))),
    };
    if game_running() {
        return Err(Fail::new(Exit::Running, "the game is running: quit it, then run the setup again"));
    }
    let root = use_disc(game)?;
    out.line(&format!("the disc: {}{}", root.display(), if matches!(copy, Copy::Image(_)) { " (mounted from the image)" } else { "" }));
    out.progress(0.005, "Your copy is the US disc (BCUS98107)");

    // Apple's Command Line Tools (the compiler and git): their installer is
    // a window of Apple's the player has to click through
    out.progress(0.01, "Checking Apple's Command Line Tools");
    if !clt_installed() {
        let opened = install_clt().is_ok();
        out.progress(
            0.01,
            if opened {
                "Install Apple's Command Line Tools in the window that opened, then run the setup again"
            } else {
                "Install Apple's Command Line Tools (xcode-select --install in Terminal), then run the setup again"
            },
        );
        return Err(Fail::new(Exit::NeedsTools, "Apple's Command Line Tools are not installed: their installer needs you (it was opened); run the setup again when it has finished"));
    }

    // the install's own sources: brought up to date (a checkout of the
    // player's own, like a developer's, is left as it is)
    let st = load_settings_file();
    let own = src_dir();
    if let Some(root) = port_root(&st).filter(|r| r.join("tools/rebuild.sh").is_file()) {
        let same = root.canonicalize().ok().zip(own.canonicalize().ok()).is_some_and(|(a, b)| a == b);
        if same && root.join(".git").exists() {
            out.progress(0.02, "Updating the port's sources");
            out.line(&format!("$ git -C {} pull --ff-only", root.display()));
            let sink_out: LineSink = {
                let o = Out::default();
                std::sync::Arc::new(move |l: &str| o.line(l))
            };
            let child = pull_command(&root).spawn().map_err(|e| Fail::new(Exit::Network, format!("git: {e}")))?;
            let code = pump_child(child, sink_out, None).join().unwrap_or(-1);
            if code != 0 {
                // offline, or the checkout went its own way: build what is here
                out.line(&format!("could not update the sources (git exited with {code}): installing from the ones already here"));
            }
        } else {
            out.line(&format!("the port's sources: {} (left as they are)", root.display()));
        }
    }

    // the install itself: tools/install.sh, as the window's "Install the game"
    let InstallJob { mut cmd, at, first, log } = install_job().map_err(|e| match e {
        InstallError::Clt => Fail::new(Exit::NeedsTools, "Apple's Command Line Tools are not installed"),
        InstallError::NoDisc => Fail::new(Exit::GameCopy, "the disc is not mounted"),
        InstallError::Other(m) => Fail::new(Exit::Build, m),
    })?;
    // the game's v1.7 update already in the online folder (downloaded by an
    // earlier online start) and its menus not built: they follow the campaign
    let online_after = online_menus_pending(&load_settings_file(), &at);
    let span = if online_after { ONLINE_FROM - INSTALL_FROM } else { INSTALL_SPAN };
    out.progress(INSTALL_FROM, "Starting the install");
    out.line(&first);
    let (code, step, tail) = run_tracked(out, &mut cmd, Tracker::default(), INSTALL_FROM, span, log).map_err(|e| Fail::new(Exit::Build, e))?;
    if code != 0 {
        let exit = classify_failure(&tail, step);
        return Err(Fail::new(exit, stopped("the install", code, step, &tail, &install_log_path())));
    }
    let b = resolve_builds(&at);
    if !b.ok {
        return Err(Fail::new(Exit::Build, format!("the install finished, but the game is not all there in {}: {}", b.root, b.missing.join(", "))));
    }
    out.line(&format!("the game: {}", b.eboot.as_deref().unwrap_or(&b.root)));
    if online_after && online_menus_pending(&load_settings_file(), &at) {
        // the campaign is ready whatever happens here: a failure is said,
        // and --play-online builds them again
        if let Err(f) = build_online_menus(out, ONLINE_FROM, 0.99 - ONLINE_FROM) {
            out.line(&format!("WARNING: the online menus were not built: {}", f.message()));
            out.progress(1.0, &format!("Ready to play. The online menus were not built ({}): starting Online tries again", first_line(f.message())));
            return Ok(());
        }
        out.progress(1.0, "Ready to play, offline and online");
        return Ok(());
    }
    out.progress(1.0, "Ready to play");
    Ok(())
}

/// "the install stopped in its lift step (exit 1): <its last ERROR: line>",
/// or the log's path when it printed none.
fn stopped(what: &str, code: i32, step: Option<&str>, tail: &str, log: &Path) -> String {
    let at = step.map(|s| format!(" in its {s} step")).unwrap_or_default();
    match tail.lines().rev().find(|l| l.starts_with("ERROR:")).map(|l| l.trim_start_matches("ERROR:").trim()) {
        Some(e) => format!("{what} stopped{at} (exit {code}): {e}"),
        None => format!("{what} stopped{at} (exit {code}); its log: {}", log.display()),
    }
}

/// Run `cmd` (its output piped) in its own process group, as from the
/// window, to its end: its lines printed and written to `log`, and read by
/// `tracker` into `progress` lines from `from` over `span`. Its exit code
/// (-1 for a signal), the step it was in, and its last lines.
fn run_tracked(out: &Out, cmd: &mut Command, tracker: Tracker, from: f64, span: f64, log: Option<fs::File>) -> Result<(i32, Option<&'static str>, String), String> {
    let started = std::time::Instant::now();
    let tracker = std::sync::Arc::new(Mutex::new(tracker));
    let tail = std::sync::Arc::new(Tail::default());
    let shown = std::sync::Arc::new(Out::default());
    *shown.last.lock().unwrap() = out.at();
    let sink: LineSink = {
        let (tracker, tail, shown) = (tracker.clone(), tail.clone(), shown.clone());
        std::sync::Arc::new(move |line: &str| {
            shown.line(line);
            tail.push(line);
            let now = started.elapsed().as_secs_f64();
            let mut t = tracker.lock().unwrap();
            let new_step = t.feed(line, now);
            let f = from + span * t.fraction(now);
            // a new step always; otherwise every half percent
            if new_step || f - shown.at() >= 0.005 {
                shown.progress(f, t.text());
            }
        })
    };
    // a stop reaches all of it
    own_process_group(cmd);
    let child = cmd.spawn().map_err(|e| format!("zsh: {e}"))?;
    let code = pump_child(child, sink, log).join().unwrap_or(-1);
    *out.last.lock().unwrap() = shown.at();
    let step = tracker.lock().unwrap().step();
    Ok((code, step, tail.text()))
}

// ---------------------------------------------------------------------------
// The v1.7 menus (online play)
// ---------------------------------------------------------------------------

/// What a failed online-menus build's output says went wrong: the network
/// and the disk as for the install; anything else is the build's.
fn online_failure(log: &str, step: Option<&str>) -> Exit {
    match classify_failure(log, step.or(Some("build"))) {
        e @ (Exit::Network | Exit::Disk | Exit::Running) => e,
        _ => Exit::OnlineBuild,
    }
}

/// The game has downloaded its v1.7 update into the online folder, and the
/// v1.7 menus are not built from it yet (in the port folder `root`).
fn online_menus_pending(s: &Settings, root: &Path) -> bool {
    online_update_eboot(s).is_file() && resolve_builds(root).eboot_v17.is_none()
}

/// Build the v1.7 menus from the update in the online folder, as Home's
/// "Prepare the online menus" (online_menus_job), with `progress` lines from
/// `from` over `span` and its output in online-menus.log. The caller holds
/// the install lock.
fn build_online_menus(out: &Out, from: f64, span: f64) -> Result<(), Fail> {
    let st = load_settings_file();
    let (mut cmd, first) = online_menus_job(&st).map_err(|m| Fail::new(Exit::OnlineBuild, m))?;
    out.progress(from, "Building the online menus from the game's v1.7 update (once, a few minutes)");
    out.line(&first);
    let log_path = online_menus_log_path();
    let log = fs::File::create(&log_path).ok();
    let (code, step, tail) = run_tracked(out, &mut cmd, Tracker::online(), from, span, log).map_err(|m| Fail::new(Exit::OnlineBuild, m))?;
    if code != 0 {
        return Err(Fail::new(online_failure(&tail, step), stopped("the online menus' build", code, step, &tail, &log_path)));
    }
    let root = port_root(&st).ok_or_else(|| Fail::new(Exit::NotInstalled, "no port folder"))?;
    match resolve_builds(&root).eboot_v17 {
        Some(e) => {
            out.line(&format!("the online menus: {e}"));
            Ok(())
        }
        None => Err(Fail::new(Exit::OnlineBuild, format!("the online menus' build finished, but {} has no build/{TITLE_ID}_eboot_v17", root.display()))),
    }
}

// ---------------------------------------------------------------------------
// --play (RECOMPLAUNCHER=1)
// ---------------------------------------------------------------------------

/// `--play [--game <disc>]` started by RecompLauncher: the window's Play
/// (the same checks, the save backup, the settings, last.log), then this
/// process becomes the game. Returns only when it could not start (the code
/// to exit with).
pub fn play(game: Option<&Path>) -> i32 {
    if !cfg!(target_os = "macos") {
        return fail(Fail::new(Exit::Unsupported, format!("the game runs on macOS only; this is {}", std::env::consts::OS)));
    }
    if let Some(g) = game {
        // checked before anything is written
        if let Err(f) = check_copy(g) {
            return fail(f);
        }
    }
    match install_lock() {
        Ok(Some(lock)) => drop(lock),
        Ok(None) => return fail(Fail::new(Exit::Running, "the game is being installed: wait for the setup to finish")),
        Err(_) => {}
    }
    if game_running() {
        return fail(Fail::new(Exit::Running, "the game is already running"));
    }
    if let Some(g) = game {
        if let Err(f) = use_disc(g) {
            return fail(f);
        }
    }
    let s = load_settings_file();
    let mut l = match prepare_launch(&s, false, false) {
        Ok(l) => l,
        Err(e) => return fail(launch_fail(e)),
    };
    println!("{} -> {}", l.argv.join(" "), l.log_path.display());
    if let Some(e) = &l.backup_error {
        println!("the saves' backup before playing failed: {e}");
    }
    keep_awake(&s);
    exec(&mut l.cmd, &l.argv[0])
}

/// A headless start that could not start: said on stderr and stdout, and the
/// code to exit with.
fn fail(f: Fail) -> i32 {
    eprintln!("ERROR: {}", f.message());
    println!("ERROR: {}", f.message());
    f.exit.code()
}

fn launch_fail(e: LaunchError) -> Fail {
    match e {
        LaunchError::NoDisc => Fail::new(Exit::GameCopy, format!("no {TITLE_ID} disc: pick your copy again")),
        LaunchError::WrongDisc(m) => Fail::new(Exit::GameCopy, m),
        LaunchError::UpdateFiles => Fail::new(Exit::UpdateFiles, "the v1.7 update's files are in the play folder: open the Resistance Launcher once, it moves them aside"),
        LaunchError::NotBuilt(m) => Fail::new(Exit::NotInstalled, format!("the game is not installed yet ({m}): run the setup")),
        LaunchError::OnlineId => Fail::new(Exit::OnlineId, NEEDS_ONLINE_ID),
        LaunchError::OnlineBuild => Fail::new(Exit::OnlineBuild, "the v1.7 update is downloaded and the online menus are not built from it yet"),
        LaunchError::Other(m) => Fail::new(Exit::Build, m),
    }
}

/// Keep the Mac awake for exactly as long as this process (the game) lives,
/// when the setting asks for it.
fn keep_awake(s: &Settings) {
    if s.keep_awake && Path::new("/usr/bin/caffeinate").is_file() {
        let _ = Command::new("/usr/bin/caffeinate")
            .args(["-i", "-w", &std::process::id().to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}

// ---------------------------------------------------------------------------
// --play-online (RECOMPLAUNCHER=1)
// ---------------------------------------------------------------------------

const NEEDS_ONLINE_ID: &str = "online play needs your online ID, set once in the Resistance Launcher window: type it under Online (PS Rewired) on Home, close the window, then start Online again";

/// What the player does the first time, when the disc's menus start so the
/// game can download its update.
pub(crate) const FIRST_ONLINE: &str = "Online, first time: the game starts with the disc's menus. Choose Multiplayer > Online Game: the game downloads its v1.7 update (PS Rewired's servers require it) into the online folder. Quit the game when it has finished; the next start of Online builds the online menus from it (once, a few minutes), then plays online.";

/// The launcher's own window, for the online ID (`open` on this .app, so a
/// window that is already open comes to the front). False when this is not
/// a bundled app (development builds, tests) or it could not be opened.
fn open_launcher_window() -> bool {
    if !cfg!(target_os = "macos") {
        return false;
    }
    let Some(bundle) = app_icon::app_bundle() else { return false };
    Command::new("/usr/bin/open")
        .arg(&bundle)
        .args(["--args", "--view", "home"])
        .env_remove("RECOMPLAUNCHER")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// `--play-online [--game <disc>]` started by RecompLauncher: Home's Play
/// online (the same checks and online environment, prepare_online_launch),
/// then this process becomes the game. With the update downloaded and the
/// v1.7 menus not built, they are built first. Returns only when it could
/// not start (the code to exit with).
pub fn play_online(game: Option<&Path>) -> i32 {
    if !cfg!(target_os = "macos") {
        return fail(Fail::new(Exit::Unsupported, format!("the game runs on macOS only; this is {}", std::env::consts::OS)));
    }
    if let Some(g) = game {
        // checked before anything is written
        if let Err(f) = check_copy(g) {
            return fail(f);
        }
    }
    // nothing happens online without the ID (typed in the window, never here)
    if !valid_online_id(load_settings_file().online_id.trim()) {
        let opened = open_launcher_window();
        return fail(Fail::new(Exit::OnlineId, if opened { format!("{NEEDS_ONLINE_ID} (the window has opened)") } else { NEEDS_ONLINE_ID.to_string() }));
    }
    let lock = match install_lock() {
        Ok(Some(lock)) => Some(lock),
        Ok(None) => return fail(Fail::new(Exit::Running, "the game is being installed, or its online menus built: wait for that to finish")),
        Err(_) => None,
    };
    if game_running() {
        return fail(Fail::new(Exit::Running, "the game is already running"));
    }
    if let Some(g) = game {
        if let Err(f) = use_disc(g) {
            return fail(f);
        }
    }
    let s = load_settings_file();
    if let Some(root) = port_root(&s).filter(|r| resolve_builds(r).ok) {
        if online_menus_pending(&s, &root) {
            let out = Out::default();
            if let Err(f) = build_online_menus(&out, 0.0, 0.99) {
                out.progress(out.at(), &format!("The online menus could not be built: {}", first_line(f.message())));
                return fail(f);
            }
            out.progress(1.0, "The online menus are built: starting online play");
        }
    }
    drop(lock);
    let mut l = match prepare_online_launch(&load_settings_file(), &input_env(), false) {
        Ok(l) => l,
        Err(e) => return fail(launch_fail(e)),
    };
    println!("{} -> {}", l.argv.join(" "), l.log_path.display());
    if let Some(m) = &l.mounted {
        println!("the disc: mounted at {m}");
    }
    if !l.v17 {
        // the game's own window takes over now (its output goes to the
        // online folder's last.log): said before it does
        println!("{FIRST_ONLINE}");
        eprintln!("{FIRST_ONLINE}");
    }
    keep_awake(&s);
    exec(&mut l.cmd, &l.argv[0])
}

#[cfg(unix)]
fn exec(cmd: &mut Command, what: &str) -> i32 {
    use std::os::unix::process::CommandExt;
    let e = cmd.exec(); // returns only on failure
    eprintln!("ERROR: {what}: {e}");
    println!("ERROR: {what}: {e}");
    Exit::NotInstalled.code()
}

#[cfg(not(unix))]
fn exec(cmd: &mut Command, what: &str) -> i32 {
    match cmd.status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("ERROR: {what}: {e}");
            Exit::NotInstalled.code()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("rl-recomplauncher-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn command_line_modes() {
        let p = |v: &[&str], env: bool| mode(&args(v), env);
        assert_eq!(p(&["l"], false), Ok(Mode::Window { game: None, online: false }));
        assert_eq!(p(&["l", "--prepare", "--game", "/a b/x.iso"], false), Ok(Mode::Prepare { game: Some("/a b/x.iso".into()) }));
        assert_eq!(p(&["l", "--prepare", "--game=/d"], true), Ok(Mode::Prepare { game: Some("/d".into()) }));
        assert_eq!(p(&["l", "--game", "/d", "--prepare"], false), Ok(Mode::Prepare { game: Some("/d".into()) }));
        assert_eq!(p(&["l", "--prepare"], false), Ok(Mode::Prepare { game: None }));
        assert_eq!(p(&["l", "--prepare", "--game="], false), Ok(Mode::Prepare { game: None }));
        assert!(p(&["l", "--prepare", "--game"], false).is_err());
        assert!(p(&["l", "--prepare", "--game", "--play"], false).is_err());
        assert!(p(&["l", "--prepare", "--play"], true).is_err());
        // --play is headless only when RecompLauncher started it
        assert_eq!(p(&["l", "--play"], false), Ok(Mode::Window { game: None, online: false }));
        assert_eq!(p(&["l", "--play", "--game", "/d"], false), Ok(Mode::Window { game: Some("/d".into()), online: false }));
        assert_eq!(p(&["l", "--play", "--game", "/d"], true), Ok(Mode::PlayHeadless { game: Some("/d".into()) }));
        assert_eq!(p(&["l", "--play", "--exit-with-game", "-psn_0_123"], true), Ok(Mode::PlayHeadless { game: None }));
        // without --play or --prepare, RECOMPLAUNCHER=1 changes nothing
        assert_eq!(p(&["l", "--view", "controls"], true), Ok(Mode::Window { game: None, online: false }));
        // the program name is not an argument
        assert_eq!(p(&["--prepare"], true), Ok(Mode::Window { game: None, online: false }));
        // online play: alone or after --play (the Online tile adds it to start.args)
        assert_eq!(p(&["l", "--play-online"], true), Ok(Mode::PlayOnlineHeadless { game: None }));
        assert_eq!(p(&["l", "--play", "--game", "/d", "--play-online"], true), Ok(Mode::PlayOnlineHeadless { game: Some("/d".into()) }));
        assert_eq!(p(&["l", "--play-online", "--game=/d"], true), Ok(Mode::PlayOnlineHeadless { game: Some("/d".into()) }));
        // without RecompLauncher: the window, which starts online play itself
        assert_eq!(p(&["l", "--play-online"], false), Ok(Mode::Window { game: None, online: true }));
        assert_eq!(p(&["l", "--play", "--game", "/d", "--play-online"], false), Ok(Mode::Window { game: Some("/d".into()), online: true }));
        // the offline tile's --offline is the offline game, as --play alone
        assert_eq!(p(&["l", "--play", "--game", "/d", "--offline"], true), Ok(Mode::PlayHeadless { game: Some("/d".into()) }));
        assert_eq!(p(&["l", "--play", "--offline"], false), Ok(Mode::Window { game: None, online: false }));
        assert!(p(&["l", "--play", "--offline", "--play-online"], true).is_err());
        assert!(p(&["l", "--prepare", "--play-online"], true).is_err());
        assert!(p(&["l", "--play-online", "--game"], true).is_err());
        assert_eq!(p(&["l", "--prepare", "--offline", "--game", "/d"], true), Ok(Mode::Prepare { game: Some("/d".into()) }));
    }

    #[test]
    fn progress_lines() {
        assert_eq!(progress_line(0.4, "Unpacking"), "progress 0.400 Unpacking");
        assert_eq!(progress_line(-1.0, " a\nb "), "progress 0.000 a b");
        assert_eq!(progress_line(7.0, "x"), "progress 1.000 x");
        assert_eq!(progress_line(f64::NAN, "x"), "progress 0.000 x");
        // as RecompLauncher reads them: "progress", the number, the text
        let l = progress_line(0.123456, "Compiling the game");
        let mut w = l.splitn(3, ' ');
        assert_eq!(w.next(), Some("progress"));
        assert!((w.next().unwrap().parse::<f32>().unwrap() - 0.123).abs() < 1e-6);
        assert_eq!(w.next(), Some("Compiling the game"));
        // plain lines never pass for progress lines
        assert_eq!(log_line("progress 0.5 fake"), "  progress 0.5 fake");
        assert_eq!(log_line("\x1b[1m==> build\x1b[0m"), "==> build");
        assert_eq!(log_line("progressive"), "progressive");
    }

    #[test]
    fn progress_only_goes_up() {
        let out = Out::default();
        assert_eq!(out.progress(0.3, "a"), 0.3);
        assert_eq!(out.progress(0.1, "b"), 0.3);
        assert_eq!(out.progress(2.0, "c"), 1.0);
    }

    #[test]
    fn install_output_maps_to_rising_fractions() {
        // tools/install.sh's output on a first install, roughly
        let lines = [
            "$ tools/install.sh   # in /x/src",
            "==> sources (the port, from GitHub)",
            "==> deps (the build tools and libraries: tools/bootstrap_deps.sh)",
            "==> [1/8] cmake",
            "==> [5/8] sdl2",
            "\x1b[1m==> toolkit\x1b[0m",
            "\x1b[1m==> done\x1b[0m",
            "==> venv (the Python environment)",
            "\x1b[1m==> decrypt\x1b[0m",
            "\x1b[1m==> done\x1b[0m",
            "\x1b[1m==> analyse\x1b[0m",
            "\x1b[1m==> lift (EBOOT, then game.self)\x1b[0m",
            "lifting...",
            "\x1b[1m==> spu (job-manager policy module, WWS job modules, raw-SPU programs)\x1b[0m",
            "\x1b[1m==> build\x1b[0m",
            "[1/400] Building CXX object a.o",
            "[400/400] Linking CXX executable ResistanceRecomp",
            "[1/300] Building CXX object b.o",
            "[150/300] Building CXX object c.o",
            "[300/300] Linking CXX executable ResistanceRecomp",
            "\x1b[1m==> done\x1b[0m",
            "==> done",
        ];
        let mut t = Tracker::default();
        let mut last = -1.0;
        let mut steps = Vec::new();
        for (i, l) in lines.iter().enumerate() {
            if t.feed(l, i as f64 * 10.0) {
                steps.push(t.step().unwrap());
            }
            let f = t.fraction(i as f64 * 10.0);
            assert!((0.0..=1.0).contains(&f), "{l}: {f}");
            assert!(f >= last, "{l}: {f} < {last}");
            last = f;
        }
        assert_eq!(steps, ["sources", "deps", "toolkit", "venv", "decrypt", "analyse", "lift", "spu", "build"]);
        assert!((last - 1.0).abs() < 1e-9, "{last}");
        // the deps' counter and ninja's move a step along
        let mut t = Tracker::default();
        t.feed("==> deps (x)", 0.0);
        let a = t.fraction(0.0);
        t.feed("==> [5/8] ffmpeg", 0.0);
        assert!(t.fraction(0.0) > a);
        // a step already passed does not take the bar back
        let mut t = Tracker::default();
        t.feed("==> lift", 0.0);
        let f = t.fraction(0.0);
        assert!(!t.feed("==> toolkit", 1.0));
        assert_eq!(t.step(), Some("lift"));
        assert!(t.fraction(1.0) >= f);
        // time alone never finishes a step
        let mut t = Tracker::default();
        t.feed("==> build", 0.0);
        let before: f64 = STEPS[..8].iter().map(|s| s.1).sum::<f64>() / STEPS.iter().map(|s| s.1).sum::<f64>();
        assert!(t.fraction(1e9) < 1.0 && t.fraction(1e9) > before);
        // ninja's counter outside the build step means nothing
        let mut t = Tracker::default();
        t.feed("==> lift", 0.0);
        t.feed("[399/400] x", 0.0);
        assert_eq!(t.fraction(0.0), Tracker { cur: Some(6), ..Default::default() }.fraction(0.0));
    }

    #[test]
    fn failures_map_to_exit_codes() {
        assert_eq!(classify_failure("ERROR: not enough free disk space: 2 GB free", Some("deps")), Exit::Disk);
        assert_eq!(classify_failure("ld: write failed: No space left on device", Some("build")), Exit::Disk);
        assert_eq!(classify_failure("fatal: unable to access 'https://github.com/x': Could not resolve host", Some("toolkit")), Exit::Network);
        assert_eq!(classify_failure("ERROR: could not download the port's sources. Check the internet connection and try again.", Some("sources")), Exit::Network);
        assert_eq!(classify_failure("ERROR: another install is already running in /x (process 12)", Some("deps")), Exit::Running);
        assert_eq!(classify_failure("ERROR: the disc is not mounted: mount your ...", Some("venv")), Exit::GameCopy);
        assert_eq!(classify_failure("error: use of undeclared identifier", Some("build")), Exit::Build);
        assert_eq!(classify_failure("Traceback ...", Some("lift")), Exit::Build);
        assert_eq!(classify_failure("fatal: something", Some("sources")), Exit::Network);
        assert_eq!(classify_failure("", None), Exit::Network);
    }

    #[test]
    fn exit_codes_are_distinct_and_in_the_profile() {
        let mut codes: Vec<i32> = Exit::ALL.iter().map(|e| e.code()).collect();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), Exit::ALL.len());
        assert_eq!(Exit::Ok.code(), 0);
        // recomplauncher.json at the repository's root explains every one
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../recomplauncher.json");
        let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        let m = v["exit_codes"].as_object().unwrap();
        assert_eq!(m["0"], "ok");
        for e in Exit::ALL {
            let t = m.get(&e.code().to_string()).and_then(|t| t.as_str()).unwrap_or_else(|| panic!("{e:?} not in exit_codes"));
            assert!(!t.trim().is_empty());
        }
        // the prepare and the start the profile names are the launcher's own
        let prog = |k: &str| v[k]["program"][0].as_str().unwrap().to_string();
        for k in ["prepare", "start"] {
            assert!(prog(k).ends_with("Resistance Launcher.app/Contents/MacOS/resistance-launcher"), "{}", prog(k));
        }
        assert_eq!(v["prepare"]["args"], serde_json::json!(["--prepare", "--game", "{game}"]));
        assert_eq!(v["start"]["args"][0], "--play");
    }

    /// The profile's two tiles: the offline game and online play, each with
    /// its own saves, and their arguments (after start.args, as RecompLauncher
    /// passes them) start what the tile says, headless only under it.
    #[test]
    fn profile_tiles_start_offline_and_online() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../recomplauncher.json");
        let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        let games = v["games"].as_array().unwrap();
        assert_eq!(games.len(), 2);
        let start: Vec<String> = v["start"]["args"].as_array().unwrap().iter().map(|a| a.as_str().unwrap().replace("{game}", "/Volumes/RES")).collect();
        let argv = |g: &serde_json::Value| -> Vec<String> {
            let mut a = vec!["resistance-launcher".to_string()];
            a.extend(start.iter().cloned());
            a.extend(g["args"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()));
            a
        };
        let disc = Some(PathBuf::from("/Volumes/RES"));
        let (off, on) = (&games[0], &games[1]);
        assert_eq!(off["id"], "offline");
        assert_eq!(on["id"], "online");
        assert!(on["name"].as_str().unwrap().contains("Online (PS Rewired)"));
        assert_eq!(mode(&argv(off), true), Ok(Mode::PlayHeadless { game: disc.clone() }));
        assert_eq!(mode(&argv(on), true), Ok(Mode::PlayOnlineHeadless { game: disc.clone() }));
        assert_eq!(mode(&argv(on), false), Ok(Mode::Window { game: disc.clone(), online: true }));
        // the online folder's own saves (its cwd-relative gamedata), never the offline ones
        let saves = |g: &serde_json::Value| g["saves"][0].as_str().unwrap().to_string();
        assert_eq!(saves(off), v["saves"][0].as_str().unwrap());
        assert!(saves(on).ends_with("/resistance-recomp/online/gamedata/dev_hdd0/home/00000001/savedata"), "{}", saves(on));
        assert_eq!(saves(on).replace("/online/", "/"), saves(off));
        // the online ID action opens the folder the launcher app is in
        let act = v["actions"].as_array().unwrap().iter().find(|a| a["id"] == "online_id").unwrap();
        let folder = act["open"].as_str().unwrap();
        assert_eq!(v["folders"][folder][0]["path"], "{install}");
        assert!(v["start"]["program"][0].as_str().unwrap().starts_with("Resistance Launcher.app/"));
        assert_eq!(v["folders"]["online"][0]["path"].as_str().unwrap(), saves(on).trim_end_matches("/gamedata/dev_hdd0/home/00000001/savedata"));
        // no setting can carry the online ID (it is typed in the window only)
        assert!(!fs::read_to_string(&p).unwrap().contains("onlineId"));
    }

    #[test]
    fn online_menus_output_maps_to_rising_fractions() {
        // online_menus_script's output with the deps step, roughly
        let lines = [
            "$ cd /x/src && tools/rebuild.sh eboot17   # the v1.7 menus, for online play",
            "==> deps",
            "==> [3/8] ninja",
            "\x1b[1m==> toolkit (the ps3recomp checkout)\x1b[0m",
            "\x1b[1m==> done\x1b[0m",
            "==> venv",
            "\x1b[1m==> eboot17 (the v1.7 menus, for online play: decrypt, analyse, lift, spu, build)\x1b[0m",
            "\x1b[1m==> v1.7 menus: decrypt\x1b[0m",
            "  from /x/online/dev_hdd0/game/BCUS98107/USRDIR/EBOOT.BIN",
            "\x1b[1m==> v1.7 menus: analyse\x1b[0m",
            "\x1b[1m==> v1.7 menus: lift\x1b[0m",
            "\x1b[1m==> v1.7 menus: spu\x1b[0m",
            "\x1b[1m==> v1.7 menus: build\x1b[0m",
            "[1/500] Building CXX object a.o",
            "[250/500] Building CXX object b.o",
            "[500/500] Linking CXX executable ResistanceRecomp",
            "\x1b[1m==> done\x1b[0m",
        ];
        let mut t = Tracker::online();
        assert_eq!(t.text(), "Starting the online menus' build");
        let (mut last, mut steps) = (-1.0, Vec::new());
        for (i, l) in lines.iter().enumerate() {
            if t.feed(l, i as f64) {
                steps.push(t.step().unwrap());
            }
            let f = t.fraction(i as f64);
            assert!((0.0..=1.0).contains(&f) && f >= last, "{l}: {f} after {last}");
            last = f;
        }
        assert_eq!(steps, ["deps", "toolkit", "venv", "decrypt", "analyse", "lift", "spu", "build"]);
        assert!((last - 1.0).abs() < 1e-9, "{last}");
        // one build target: ninja half way is half of the build step
        let mut t = Tracker::online();
        t.feed("==> v1.7 menus: build", 0.0);
        let at = |t: &Tracker| t.fraction(0.0);
        let start = at(&t);
        t.feed("[250/500] x", 0.0);
        let total: f64 = ONLINE_STEPS.iter().map(|s| s.1).sum();
        assert!((at(&t) - start - 0.5 * 35.0 / total).abs() < 1e-9);
        // the install's tracker does not take the online steps' names
        let mut t = Tracker::default();
        assert!(!t.feed("==> eboot17 (x)", 0.0));
        assert_eq!(t.text(), "Starting the install");
    }

    #[test]
    fn online_build_failures_and_messages() {
        assert_eq!(online_failure("ERROR: the downloaded EBOOT.BIN is not the tested v1.7 one", Some("decrypt")), Exit::OnlineBuild);
        assert_eq!(online_failure("error: use of undeclared identifier", Some("build")), Exit::OnlineBuild);
        assert_eq!(online_failure("Traceback ...", None), Exit::OnlineBuild);
        assert_eq!(online_failure("fatal: unable to access 'https://github.com/x': Could not resolve host", Some("toolkit")), Exit::Network);
        assert_eq!(online_failure("ld: write failed: No space left on device", Some("build")), Exit::Disk);
        let log = Path::new("/d/online-menus.log");
        assert_eq!(stopped("the online menus' build", 1, Some("lift"), "x\nERROR: boom\ny", log), "the online menus' build stopped in its lift step (exit 1): boom");
        assert_eq!(stopped("the install", 2, None, "x", log), "the install stopped (exit 2); its log: /d/online-menus.log");
        // what the player is told the first time names the menu to pick
        assert!(FIRST_ONLINE.contains("Multiplayer > Online Game") && FIRST_ONLINE.contains("next start"));
        assert!(NEEDS_ONLINE_ID.contains("Resistance Launcher window"));
        // the start's own exit codes for online play
        assert_eq!(launch_fail(LaunchError::OnlineId).exit, Exit::OnlineId);
        assert_eq!(launch_fail(LaunchError::OnlineBuild).exit, Exit::OnlineBuild);
        assert_eq!(launch_fail(LaunchError::NoDisc).exit, Exit::GameCopy);
        assert_eq!(launch_fail(LaunchError::NotBuilt("x".into())).exit, Exit::NotInstalled);
        // the install's progress leaves room for the online menus, and they end before 1.0
        const { assert!(INSTALL_FROM < ONLINE_FROM && ONLINE_FROM < 0.99) };
    }

    /// Every value the profile's settings write into launcher.json (as
    /// RecompLauncher writes a JSON value: parsed as JSON, else a string)
    /// still reads as the launcher's Settings, on a key it has, and its
    /// defaults are the launcher's own. A wrong type would make the launcher
    /// fall back to all defaults, so this guards the player's settings.
    #[test]
    fn profile_settings_fit_launcher_json() {
        use serde_json::Value;
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../recomplauncher.json");
        let v: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        let cfg = &v["config"][0];
        assert_eq!(cfg["format"], "json");
        assert!(cfg["file"][0]["path"].as_str().unwrap().ends_with("/Library/Application Support/resistance-recomp/launcher.json"));
        assert_eq!(settings_path().file_name().unwrap(), "launcher.json");
        let base = serde_json::to_value(Settings::default()).unwrap();
        let base = base.as_object().unwrap();
        let as_rl = |s: &str| serde_json::from_str::<Value>(s).unwrap_or(Value::String(s.into()));
        // never offered here: the frame rate (PS Rewired's rules for online
        // play), online play's own, the v1.7 campaign, the disc and folders
        let never = ["unlockFps", "menu60fps", "vsync", "frameClockUs", "onlineId", "voiceChat", "campaign", "extraEnv", "discPath", "discImage", "portDir", "playDir"];
        let settings = v["settings"].as_array().unwrap();
        assert!(settings.len() >= 10);
        for st in settings {
            let id = st["id"].as_str().unwrap();
            let mut writes: Vec<(String, String)> = Vec::new();
            for c in st["choices"].as_array().into_iter().flatten() {
                for (k, val) in c["set"].as_object().unwrap() {
                    writes.push((k.clone(), val.as_str().unwrap().to_string()));
                }
            }
            if let Some(sl) = st.get("slider") {
                let k = sl["key"].as_str().unwrap();
                let fmt = sl["format"].as_str().unwrap();
                for x in [sl["min"].as_f64().unwrap(), sl["max"].as_f64().unwrap()] {
                    let val = match fmt {
                        "%d" => format!("{}", x as i64),
                        "%.2f" => format!("{x:.2}"),
                        f => panic!("{id}: format {f}"),
                    };
                    writes.push((k.to_string(), val));
                }
            }
            assert!(!writes.is_empty(), "{id}");
            for (k, val) in st["defaults"].as_object().unwrap() {
                let d = as_rl(val.as_str().unwrap());
                let have = &base[k];
                let same = match (d.as_f64(), have.as_f64()) {
                    (Some(a), Some(b)) => (a - b).abs() < 1e-6,
                    _ => d == *have,
                };
                assert!(same, "{id}: default {k}={d} but the launcher's is {have}");
            }
            for (k, val) in writes {
                assert!(base.contains_key(&k), "{id}: launcher.json has no {k}");
                assert!(!never.contains(&k.as_str()), "{id}: {k} is not offered in RecompLauncher");
                let mut o = base.clone();
                o.insert(k.clone(), as_rl(&val));
                serde_json::from_value::<Settings>(Value::Object(o)).unwrap_or_else(|e| panic!("{id}: {k}={val}: {e}"));
            }
        }
    }

    fn sfo(title_id: &str) -> Vec<u8> {
        let e = |k: &str, v: &str| saves::PsfEntry { key: k.into(), fmt: saves::PSF_UTF8, data: format!("{v}\0").into_bytes(), max_len: 16 };
        saves::psf_write(&[e("TITLE_ID", title_id), e("TITLE", "RESISTANCE")])
    }

    /// A tiny ISO 9660 image: PS3_DISC.SFB and PS3_GAME/PARAM.SFO.
    fn fake_iso(path: &Path, title_id: &str) {
        let sec = 2048usize;
        let mut img = vec![0u8; 24 * sec];
        let rec = |lba: u32, size: u32, dir: bool, name: &[u8]| -> Vec<u8> {
            let len = 33 + name.len() + (name.len() + 1) % 2;
            let mut r = vec![0u8; len];
            r[0] = len as u8;
            r[2..6].copy_from_slice(&lba.to_le_bytes());
            r[6..10].copy_from_slice(&lba.to_be_bytes());
            r[10..14].copy_from_slice(&size.to_le_bytes());
            r[14..18].copy_from_slice(&size.to_be_bytes());
            r[25] = if dir { 2 } else { 0 };
            r[32] = name.len() as u8;
            r[33..33 + name.len()].copy_from_slice(name);
            r
        };
        let sfo = sfo(title_id);
        // PVD at 16; root dir at 18; PS3_GAME dir at 19; SFO at 20; SFB at 21
        img[16 * sec] = 1;
        img[16 * sec + 1..16 * sec + 6].copy_from_slice(b"CD001");
        let root = rec(18, sec as u32, true, b"\0");
        img[16 * sec + 156..16 * sec + 156 + root.len()].copy_from_slice(&root);
        img[17 * sec] = 255;
        img[17 * sec + 1..17 * sec + 6].copy_from_slice(b"CD001");
        let mut at = 18 * sec;
        for r in [rec(18, sec as u32, true, b"\0"), rec(18, sec as u32, true, b"\x01"), rec(21, 8, false, b"PS3_DISC.SFB;1"), rec(19, sec as u32, true, b"PS3_GAME")] {
            img[at..at + r.len()].copy_from_slice(&r);
            at += r.len();
        }
        let mut at = 19 * sec;
        for r in [rec(19, sec as u32, true, b"\0"), rec(18, sec as u32, true, b"\x01"), rec(20, sfo.len() as u32, false, b"PARAM.SFO;1")] {
            img[at..at + r.len()].copy_from_slice(&r);
            at += r.len();
        }
        img[20 * sec..20 * sec + sfo.len()].copy_from_slice(&sfo);
        img[21 * sec..21 * sec + 8].copy_from_slice(b".SFB\0\0\0\0");
        fs::write(path, img).unwrap();
    }

    #[test]
    fn disc_folders_and_images_are_checked() {
        let t = tmp("copy");
        // nothing there
        assert_eq!(check_copy(&t.join("nope")).unwrap_err().exit, Exit::GameCopy);
        // a folder without PS3_GAME
        assert_eq!(check_copy(&t).unwrap_err().exit, Exit::GameCopy);
        // the US disc, as a folder (or its PS3_GAME)
        let disc = t.join("disc");
        fs::create_dir_all(disc.join("PS3_GAME")).unwrap();
        fs::write(disc.join("PS3_DISC.SFB"), b".SFB").unwrap();
        fs::write(disc.join("PS3_GAME/PARAM.SFO"), sfo("BCUS98107")).unwrap();
        assert_eq!(check_copy(&disc).unwrap(), Copy::Folder(disc.clone()));
        assert_eq!(check_copy(&disc.join("PS3_GAME")).unwrap(), Copy::Folder(disc.clone()));
        // another region's disc
        fs::write(disc.join("PS3_GAME/PARAM.SFO"), sfo("BCES00001")).unwrap();
        let e = check_copy(&disc).unwrap_err();
        assert!(e.exit == Exit::GameCopy && e.message().contains("BCES00001"), "{}", e.message());
        // a disc image, read without mounting it
        let iso = t.join("Resistance (USA).iso");
        fake_iso(&iso, "BCUS98107");
        assert_eq!(iso_read(&iso, &["PS3_DISC.SFB"], 64).as_deref(), Some(&b".SFB\0\0\0\0"[..]));
        assert_eq!(iso_read(&iso, &["ps3_game", "param.sfo"], 1 << 20), Some(sfo("BCUS98107")));
        assert_eq!(iso_read(&iso, &["PS3_GAME", "NOPE.SFO"], 1 << 20), None);
        assert_eq!(iso_read(&iso, &["PARAM.SFO"], 1 << 20), None); // not at the top
        assert_eq!(iso_read(&iso, &["PS3_GAME"], 1 << 20), None); // a folder, not a file
        assert_eq!(check_copy(&iso).unwrap(), Copy::Image(iso.clone()));
        let other = t.join("other.iso");
        fake_iso(&other, "BLUS30001");
        assert!(check_copy(&other).unwrap_err().message().contains("BLUS30001"));
        // not a disc image at all
        let junk = t.join("junk.iso");
        fs::write(&junk, vec![7u8; 40 * 2048]).unwrap();
        assert_eq!(check_copy(&junk).unwrap_err().exit, Exit::GameCopy);
        fs::write(&junk, b"tiny").unwrap();
        assert_eq!(check_copy(&junk).unwrap_err().exit, Exit::GameCopy);
        let _ = fs::remove_dir_all(&t);
    }

    #[test]
    fn the_install_lock_is_exclusive() {
        // its own file here: the real one lives in the launcher's data folder
        let t = tmp("lock");
        let p = t.join("install.lock");
        let open = || fs::OpenOptions::new().create(true).truncate(false).write(true).open(&p).unwrap();
        let a = open();
        a.try_lock().unwrap();
        assert!(matches!(open().try_lock(), Err(fs::TryLockError::WouldBlock)));
        drop(a);
        open().try_lock().unwrap();
        let _ = fs::remove_dir_all(&t);
    }
}
