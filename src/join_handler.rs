use std::sync::{Arc, Mutex};

use pumpkin_plugin_api::{
    Server,
    events::{EventData, EventHandler, PlayerJoinEvent},
    scheduler,
    text::TextComponent,
};

use crate::module::{Module, MsptModule, PlayerCountModule, TpsModule, compose};

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
        let id = scheduler::schedule_repeating_task(20, 20, move |server| {
            if let Some(player) = server.get_player_by_uuid(player_id) {
                player.set_tab_list_header_footer(render_header(&server), render_footer(&server));
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

fn render_header(server: &Server) -> TextComponent {
    let modules: [&dyn Module; 2] = [&TpsModule, &PlayerCountModule];
    compose(&modules, server)
}

fn render_footer(server: &Server) -> TextComponent {
    let modules: [&dyn Module; 1] = [&MsptModule];
    compose(&modules, server)
}
