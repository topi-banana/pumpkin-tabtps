use std::{
    fs,
    path::Path,
    sync::{Arc, OnceLock, RwLock},
};

use serde::{Deserialize, Serialize};

/// Name of the config file inside the plugin's data folder.
pub const CONFIG_FILE_NAME: &str = "tabtps.toml";

/// Template written to disk on first run when no `tabtps.toml` exists. The
/// values must mirror [`Config::default`] / [`ColorConfig::default`] /
/// [`LayoutConfig::default`] / [`ActionbarConfig::default`] — if you change a
/// default, update this template too.
const DEFAULT_CONFIG_TEMPLATE: &str = "\
# TabTPS configuration — auto-generated on first run.
# Edit then run `/tabtps reload` (planned) or restart the server to apply.

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
";

/// Default tick interval (`20` ticks ≈ 1 second on a healthy server).
pub const DEFAULT_UPDATE_INTERVAL_TICKS: u32 = 20;

/// Plugin-wide configuration loaded from `tabtps.toml`. Reloadable via the
/// (planned) `/tabtps reload` command — the live values live in a shared
/// `RwLock` so reads from per-tick tasks see the latest snapshot without
/// having to be rescheduled.
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_interval_ticks: DEFAULT_UPDATE_INTERVAL_TICKS,
            colors: ColorConfig::default(),
            layout: LayoutConfig::default(),
            actionbar: ActionbarConfig::default(),
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

/// Replace the live config — used by startup load and (eventually) the reload
/// command.
pub fn replace(new_config: Config) {
    *config().write().unwrap() = new_config;
}
