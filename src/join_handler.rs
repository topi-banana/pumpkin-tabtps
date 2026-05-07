use std::sync::{Arc, Mutex};

use pumpkin_plugin_api::{
    Server,
    events::{EventData, EventHandler, PlayerJoinEvent},
    scheduler,
    text::{NamedColor, TextComponent},
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
        let id = scheduler::schedule_repeating_task(20, 20, move |server| {
            if let Some(player) = server.get_player_by_uuid(&player_id) {
                player.set_tab_list_header_footer(TextComponent::text(""), gen_footer(&server));
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

fn gen_footer(server: &Server) -> TextComponent {
    let mspt = server.get_mspt();
    let tps = server.get_tps();
    let color = match mspt {
        ..25.0 => NamedColor::Green,
        ..40.0 => NamedColor::Gold,
        _ => NamedColor::Red,
    };
    let footer = gen_text_component("TPS", &format!("{tps:.2}"), color);
    footer
        .add_child(TextComponent::text(" "))
        .add_child(gen_text_component("MSPT", &format!("{mspt:.2}"), color));
    footer
}

fn gen_text_component(name: &'static str, value: &str, color: NamedColor) -> TextComponent {
    let result = TextComponent::text(name);
    result.color_named(NamedColor::Gray);
    result.add_child({
        let sep_component = TextComponent::text(": ");
        sep_component.color_named(NamedColor::White);
        sep_component
    });
    result.add_child({
        let value_component = TextComponent::text(value);
        value_component.color_named(color);
        value_component
    });
    result
}
