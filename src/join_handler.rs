use std::sync::{Arc, Mutex};

use pumpkin_plugin_api::{
    Server,
    events::{EventData, EventHandler, PlayerJoinEvent},
    player::Player,
    scheduler,
    text::TextComponent,
};

use crate::{
    config,
    module::{Module, compose, module_by_name},
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
        let id = scheduler::schedule_repeating_task(interval, interval, move |server| {
            if let Some(player) = server.get_player_by_uuid(player_id) {
                player.set_tab_list_header_footer(
                    render_header(&server, &player),
                    render_footer(&server, &player),
                );
            } else if let Some(id) = task_slot_clone.lock().unwrap().take() {
                tracing::info!("Player gone, cancelling tab task id={id}");
                scheduler::cancel_task(id);
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
