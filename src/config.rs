use std::{
    fs,
    path::Path,
    sync::{Arc, OnceLock, RwLock},
};

use serde::{Deserialize, Serialize};

/// Name of the config file inside the plugin's data folder.
pub const CONFIG_FILE_NAME: &str = "tabtps.toml";

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_interval_ticks: DEFAULT_UPDATE_INTERVAL_TICKS,
            colors: ColorConfig::default(),
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
                    "Failed to parse TabTPS config; falling back to defaults",
                );
                Config::default()
            }
        },
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
