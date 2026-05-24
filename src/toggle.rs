use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

/// Per-display visibility flags for one player. The render loop in
/// [`join_handler`] checks each field before pushing an update, so a player
/// can suppress individual surfaces without disabling them globally.
///
/// [`join_handler`]: crate::join_handler
#[derive(Debug, Clone, Copy)]
pub struct PlayerToggles {
    pub tab: bool,
    pub actionbar: bool,
    pub bossbar: bool,
}

impl Default for PlayerToggles {
    fn default() -> Self {
        Self {
            tab: true,
            actionbar: true,
            bossbar: true,
        }
    }
}

/// Stable lookup key built from the WIT `uuid { high, low }` pair. We can't
/// name the WIT-generated `Uuid` type from outside `pumpkin-plugin-api`, so
/// callers extract the two `u64`s at the boundary instead.
pub type PlayerKey = (u64, u64);

static TOGGLES: OnceLock<Mutex<HashMap<PlayerKey, PlayerToggles>>> = OnceLock::new();

fn map() -> &'static Mutex<HashMap<PlayerKey, PlayerToggles>> {
    TOGGLES.get_or_init(Mutex::default)
}

/// Returns the active toggles for a player, defaulting to "everything on" when
/// the player has never been touched by a setter (no `/tabtps toggle` command
/// has flipped anything yet).
pub fn for_player(key: PlayerKey) -> PlayerToggles {
    map().lock().unwrap().get(&key).copied().unwrap_or_default()
}

/// Drop a player's stored overrides on disconnect so the map can't grow
/// without bound. Persistence (Phase 5) will reload them on rejoin instead.
pub fn forget(key: PlayerKey) {
    map().lock().unwrap().remove(&key);
}
