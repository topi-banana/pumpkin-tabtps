use std::sync::{Arc, Mutex};

use pumpkin_plugin_api::{
    Server,
    boss_bar::{BossBar, BossBarColor, BossBarDivision},
    events::{EventData, EventHandler, PlayerJoinEvent},
    player::Player,
    scheduler,
    text::TextComponent,
};

use crate::{
    config::{self, ColorConfig, ThemeColor, ThemeConfig},
    module::{Module, compose, module_by_name},
    toggle,
};

pub struct TabtpsJoinHandler;

impl EventHandler<PlayerJoinEvent> for TabtpsJoinHandler {
    fn handle(
        &self,
        _server: Server,
        event: EventData<PlayerJoinEvent>,
    ) -> EventData<PlayerJoinEvent> {
        tracing::info!("Player joined: {}", event.player.get_name());
        let player_id = event.player.get_id();

        let task_slot: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
        let task_slot_clone = task_slot.clone();
        let interval = config::current_update_interval_ticks();
        let key: toggle::PlayerKey = (player_id.high, player_id.low);
        let mut bossbar: Option<BossBar> = None;
        let id = scheduler::schedule_repeating_task(interval, interval, move |server| {
            if let Some(player) = server.get_player_by_uuid(player_id) {
                let toggles = toggle::for_player(key);
                if toggles.tab {
                    player.set_tab_list_header_footer(
                        render_header(&server, &player),
                        render_footer(&server, &player),
                    );
                }
                if toggles.actionbar
                    && let Some(text) = render_actionbar(&server, &player)
                {
                    player.show_actionbar(text);
                }
                update_bossbar(&server, player, &mut bossbar, toggles.bossbar);
            } else {
                if let Some(bb) = bossbar.take() {
                    bb.remove_all();
                }
                toggle::forget(key);
                if let Some(id) = task_slot_clone.lock().unwrap().take() {
                    tracing::info!("Player gone, cancelling tab task id={id}");
                    scheduler::cancel_task(id);
                }
            }
        });
        *task_slot.lock().unwrap() = Some(id);
        tracing::info!("Tab task scheduled (id={id})");

        event
    }
}

fn render_header(server: &Server, player: &Player) -> TextComponent {
    render_slot(|cfg| &cfg.layout.header, server, player)
}

fn render_footer(server: &Server, player: &Player) -> TextComponent {
    render_slot(|cfg| &cfg.layout.footer, server, player)
}

fn render_slot(
    pick: impl FnOnce(&config::Config) -> &Vec<String>,
    server: &Server,
    player: &Player,
) -> TextComponent {
    let cfg = config::config().read().unwrap();
    let modules: Vec<&'static dyn Module> = pick(&cfg)
        .iter()
        .filter_map(|name| module_by_name(name))
        .collect();
    compose(&modules, server, player)
}

fn render_actionbar(server: &Server, player: &Player) -> Option<TextComponent> {
    let cfg = config::config().read().unwrap();
    if !cfg.actionbar.enabled || cfg.actionbar.modules.is_empty() {
        return None;
    }
    let modules: Vec<&'static dyn Module> = cfg
        .actionbar
        .modules
        .iter()
        .filter_map(|name| module_by_name(name))
        .collect();
    if modules.is_empty() {
        return None;
    }
    Some(compose(&modules, server, player))
}

fn update_bossbar(server: &Server, player: Player, bossbar: &mut Option<BossBar>, allowed: bool) {
    let cfg = config::config().read().unwrap();
    if !allowed || !cfg.bossbar.enabled || cfg.bossbar.modules.is_empty() {
        if let Some(bb) = bossbar.take() {
            bb.remove_all();
        }
        return;
    }
    let modules: Vec<&'static dyn Module> = cfg
        .bossbar
        .modules
        .iter()
        .filter_map(|name| module_by_name(name))
        .collect();
    if modules.is_empty() {
        if let Some(bb) = bossbar.take() {
            bb.remove_all();
        }
        return;
    }

    let mspt = server.get_mspt();
    let progress = mspt_progress(mspt);
    let color = bossbar_color_for_mspt(mspt, &cfg.colors, &cfg.theme);
    let title = compose(&modules, server, &player);
    drop(cfg);

    match bossbar {
        None => {
            let bb = BossBar::new(title, color, BossBarDivision::Notches20);
            bb.add_player(player);
            bb.set_health(progress);
            *bossbar = Some(bb);
        }
        Some(bb) => {
            bb.set_title(title);
            bb.set_health(progress);
            bb.set_color(color);
        }
    }
}

fn mspt_progress(mspt: f64) -> f32 {
    (mspt / 50.0).clamp(0.0, 1.0) as f32
}

fn bossbar_color_for_mspt(mspt: f64, colors: &ColorConfig, theme: &ThemeConfig) -> BossBarColor {
    let bucket = if mspt < colors.mspt_green_max {
        theme.mspt_good
    } else if mspt < colors.mspt_gold_max {
        theme.mspt_warn
    } else {
        theme.mspt_bad
    };
    theme_to_bossbar(bucket)
}

/// Best-fit mapping from the 16-colour NamedColor palette down to the boss
/// bar's 7-colour palette. Greys collapse to white; navy / aqua collapse to
/// blue; dark-red / red to red; gold / yellow to yellow; etc.
fn theme_to_bossbar(c: ThemeColor) -> BossBarColor {
    use BossBarColor as B;
    use ThemeColor as T;
    match c {
        T::Black | T::Gray | T::DarkGray | T::White => B::White,
        T::DarkBlue | T::Blue | T::DarkAqua | T::Aqua => B::Blue,
        T::DarkGreen | T::Green => B::Green,
        T::DarkRed | T::Red => B::Red,
        T::DarkPurple => B::Purple,
        T::LightPurple => B::Pink,
        T::Gold | T::Yellow => B::Yellow,
    }
}
