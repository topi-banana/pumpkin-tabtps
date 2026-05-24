use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock, RwLock},
};

use pumpkin_plugin_api::text::NamedColor;
use serde::{Deserialize, Serialize};

/// Name of the config file inside the plugin's data folder.
pub const CONFIG_FILE_NAME: &str = "tabtps.toml";

/// Template written to disk on first run when no `tabtps.toml` exists. The
/// values must mirror [`Config::default`] / [`ColorConfig::default`] /
/// [`LayoutConfig::default`] / [`ActionbarConfig::default`] /
/// [`BossbarConfig::default`] / [`ThemeConfig::default`] — if you change a
/// default, update this template too.
const DEFAULT_CONFIG_TEMPLATE: &str = "\
# TabTPS configuration — auto-generated on first run.
# Edit then run `/tabtps reload` (or restart the server) to apply.

# Tab list refresh cadence. 20 ticks ≈ 1 second.
# Read at PlayerJoinEvent time, so existing players keep their previous
# cadence until they rejoin. `0` is rejected.
update_interval_ticks = 20

[colors]
# MSPT strictly below this value renders green.
mspt_green_max = 25.0
# ... below this renders gold; everything else renders red.
mspt_gold_max = 40.0

[layout]
# Modules rendered into the tab list, in order. Empty lists hide the slot.
# Available names: tps, mspt, player_count, ping
header = [\"tps\", \"player_count\"]
footer = [\"mspt\", \"ping\"]

[actionbar]
# Set `enabled = false` to suppress the action bar entirely.
enabled = true
modules = [\"tps\", \"mspt\"]

[bossbar]
# Set `enabled = false` to suppress the boss bar entirely.
# Progress is mapped from MSPT (`mspt / 50.0` clamped to [0, 1]) and the
# bar colour follows the [colors] MSPT thresholds (green/yellow/red).
enabled = true
modules = [\"tps\", \"mspt\", \"ping\"]

[theme]
# Named-colour overrides for every value rendered into a tab list, action
# bar, or boss bar. Valid colour names: black, dark_blue, dark_green,
# dark_aqua, dark_red, dark_purple, gold, gray, dark_gray, blue, green,
# aqua, red, light_purple, yellow, white.
mspt_good = \"green\"
mspt_warn = \"gold\"
mspt_bad = \"red\"
ping_good = \"green\"
ping_warn = \"gold\"
ping_bad = \"red\"
label = \"gray\"
separator = \"white\"
player_count = \"white\"
";

/// Default tick interval (`20` ticks ≈ 1 second on a healthy server).
pub const DEFAULT_UPDATE_INTERVAL_TICKS: u32 = 20;

/// Plugin-wide configuration loaded from `tabtps.toml`. Reloadable via the
/// `/tabtps reload` command — the live values live in a shared `RwLock` so
/// reads from per-tick tasks see the latest snapshot without having to be
/// rescheduled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Tick cadence for the tab list refresh task. Read by [`join_handler`]
    /// each time a player joins; changes only affect players who rejoin (or,
    /// in the future, are picked up by `/tabtps reload`).
    ///
    /// [`join_handler`]: crate::join_handler
    pub update_interval_ticks: u32,

    pub colors: ColorConfig,

    pub layout: LayoutConfig,

    pub actionbar: ActionbarConfig,

    pub bossbar: BossbarConfig,

    pub theme: ThemeConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_interval_ticks: DEFAULT_UPDATE_INTERVAL_TICKS,
            colors: ColorConfig::default(),
            layout: LayoutConfig::default(),
            actionbar: ActionbarConfig::default(),
            bossbar: BossbarConfig::default(),
            theme: ThemeConfig::default(),
        }
    }
}

/// Serde-friendly mirror of [`NamedColor`] using snake_case strings, so the
/// theme file can read `"dark_red"` instead of having to inline a Rust enum
/// import.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl ThemeColor {
    pub fn to_named(self) -> NamedColor {
        match self {
            Self::Black => NamedColor::Black,
            Self::DarkBlue => NamedColor::DarkBlue,
            Self::DarkGreen => NamedColor::DarkGreen,
            Self::DarkAqua => NamedColor::DarkAqua,
            Self::DarkRed => NamedColor::DarkRed,
            Self::DarkPurple => NamedColor::DarkPurple,
            Self::Gold => NamedColor::Gold,
            Self::Gray => NamedColor::Gray,
            Self::DarkGray => NamedColor::DarkGray,
            Self::Blue => NamedColor::Blue,
            Self::Green => NamedColor::Green,
            Self::Aqua => NamedColor::Aqua,
            Self::Red => NamedColor::Red,
            Self::LightPurple => NamedColor::LightPurple,
            Self::Yellow => NamedColor::Yellow,
            Self::White => NamedColor::White,
        }
    }
}

/// User-overridable colour palette used everywhere the renderer puts a value
/// on-screen. Defaults reproduce the prior hard-coded scheme exactly.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub mspt_good: ThemeColor,
    pub mspt_warn: ThemeColor,
    pub mspt_bad: ThemeColor,
    pub ping_good: ThemeColor,
    pub ping_warn: ThemeColor,
    pub ping_bad: ThemeColor,
    pub label: ThemeColor,
    pub separator: ThemeColor,
    pub player_count: ThemeColor,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            mspt_good: ThemeColor::Green,
            mspt_warn: ThemeColor::Gold,
            mspt_bad: ThemeColor::Red,
            ping_good: ThemeColor::Green,
            ping_warn: ThemeColor::Gold,
            ping_bad: ThemeColor::Red,
            label: ThemeColor::Gray,
            separator: ThemeColor::White,
            player_count: ThemeColor::White,
        }
    }
}

/// Boss bar display options. Progress is fixed to MSPT mode for now
/// (`progress = mspt / 50.0` clamped to `[0, 1]`); the bar colour follows the
/// [`ColorConfig`] MSPT thresholds (with `gold` mapped to the boss bar's
/// closest equivalent, `yellow`). The boss bar shares the tab list refresh
/// cadence and is created lazily on the first tick a player is reachable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BossbarConfig {
    pub enabled: bool,
    pub modules: Vec<String>,
}

impl Default for BossbarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            modules: vec!["tps".into(), "mspt".into(), "ping".into()],
        }
    }
}

/// Action bar display options. The action bar shares the tab list refresh
/// cadence (driven by [`Config::update_interval_ticks`]); set `enabled = false`
/// to suppress it entirely.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionbarConfig {
    pub enabled: bool,
    pub modules: Vec<String>,
}

impl Default for ActionbarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            modules: vec!["tps".into(), "mspt".into()],
        }
    }
}

/// Names of the modules rendered into the tab list header / footer. Order is
/// preserved; unknown names are dropped at load time with a warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    pub header: Vec<String>,
    pub footer: Vec<String>,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            header: vec!["tps".into(), "player_count".into()],
            footer: vec!["mspt".into(), "ping".into()],
        }
    }
}

/// MSPT → tab list color thresholds. A reading is **green** when strictly
/// below `mspt_green_max`, **gold** when below `mspt_gold_max`, and **red**
/// otherwise.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorConfig {
    pub mspt_green_max: f64,
    pub mspt_gold_max: f64,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            mspt_green_max: 25.0,
            mspt_gold_max: 40.0,
        }
    }
}

/// Read and parse `<data_folder>/tabtps.toml`. A missing or unparseable file
/// is reported through `tracing::error!` and the caller receives
/// [`Config::default`] so the plugin keeps running.
pub fn load_from_disk(data_folder: &Path) -> Config {
    let path = data_folder.join(CONFIG_FILE_NAME);
    match fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<Config>(&contents) {
            Ok(mut cfg) => {
                validate(&mut cfg);
                tracing::info!(path = %path.display(), "Loaded TabTPS config");
                cfg
            }
            Err(err) => {
                tracing::error!(
                    path = %path.display(),
                    error = %err,
                    "Failed to parse TabTPS config; falling back to defaults \
                     (the file is preserved so you can fix it in place)",
                );
                Config::default()
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            match fs::write(&path, DEFAULT_CONFIG_TEMPLATE) {
                Ok(()) => tracing::info!(
                    path = %path.display(),
                    "Created default TabTPS config",
                ),
                Err(write_err) => tracing::error!(
                    path = %path.display(),
                    error = %write_err,
                    "Could not write default TabTPS config; using in-memory defaults",
                ),
            }
            Config::default()
        }
        Err(err) => {
            tracing::error!(
                path = %path.display(),
                error = %err,
                "Could not read TabTPS config; falling back to defaults",
            );
            Config::default()
        }
    }
}

/// Apply sanity checks to a freshly-parsed config, replacing nonsense values
/// with their defaults. Each adjustment is logged so the operator notices.
fn validate(cfg: &mut Config) {
    if cfg.update_interval_ticks == 0 {
        tracing::warn!(
            "tabtps.toml: update_interval_ticks = 0 would spam the scheduler; \
             resetting to default ({DEFAULT_UPDATE_INTERVAL_TICKS})",
        );
        cfg.update_interval_ticks = DEFAULT_UPDATE_INTERVAL_TICKS;
    }
    drop_unknown_modules(&mut cfg.layout.header, "header");
    drop_unknown_modules(&mut cfg.layout.footer, "footer");
    drop_unknown_modules(&mut cfg.actionbar.modules, "actionbar");
    drop_unknown_modules(&mut cfg.bossbar.modules, "bossbar");
}

fn drop_unknown_modules(names: &mut Vec<String>, slot: &str) {
    names.retain(|name| {
        if crate::module::module_by_name(name).is_some() {
            true
        } else {
            tracing::warn!(
                slot,
                name = %name,
                "tabtps.toml: unknown module name in [layout] — skipping",
            );
            false
        }
    });
}

/// Current update interval as ticks, suitable for passing to
/// [`pumpkin_plugin_api::scheduler::schedule_repeating_task`].
pub fn current_update_interval_ticks() -> u64 {
    u64::from(config().read().unwrap().update_interval_ticks)
}

static CONFIG: OnceLock<Arc<RwLock<Config>>> = OnceLock::new();

/// Returns the global config handle, initialising it with defaults on first
/// access. Modules read from this lock each tick; `/tabtps reload` (future)
/// will swap the contents via [`replace`].
pub fn config() -> &'static Arc<RwLock<Config>> {
    CONFIG.get_or_init(|| Arc::new(RwLock::new(Config::default())))
}

/// Replace the live config — used by startup load and the `/tabtps reload`
/// command.
pub fn replace(new_config: Config) {
    *config().write().unwrap() = new_config;
}

static DATA_FOLDER: OnceLock<PathBuf> = OnceLock::new();

/// Remember the plugin's data folder so the `/tabtps reload` handler can find
/// `tabtps.toml` later. Called once from `on_load`.
pub fn init_data_folder(path: PathBuf) {
    let _ = DATA_FOLDER.set(path);
}

/// Re-read `tabtps.toml` and swap the live config in. Returns the new config
/// on success, or an error string suitable for showing to the command sender.
pub fn reload() -> Result<Config, &'static str> {
    let folder = DATA_FOLDER.get().ok_or("data folder not initialised yet")?;
    let new_config = load_from_disk(folder);
    replace(new_config.clone());
    Ok(new_config)
}
