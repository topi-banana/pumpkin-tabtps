use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use pumpkin::{
    plugin::{Context, EventHandler, EventPriority, player::player_join::PlayerJoinEvent},
    server::Server,
};
use pumpkin_api_macros::{plugin_impl, plugin_method, with_runtime};
use pumpkin_protocol::java::client::play::CTabList;
use pumpkin_util::text::{TextComponent, color::NamedColor};

struct MyJoinHandler;

#[with_runtime(global)]
#[async_trait]
impl EventHandler<PlayerJoinEvent> for MyJoinHandler {
    async fn handle(&self, server: &Arc<Server>, event: &PlayerJoinEvent) {
        let player = Arc::clone(&event.player);
        let server = Arc::clone(server);
        tokio::spawn(async move {
            loop {
                let nspts = server.get_tick_times_nanos_copy().await;
                let avg_mspt = nspts.iter().copied().sum::<i64>() as f64 / 100.0 / 1_000_000.0;
                let tps = if avg_mspt > 50.0 {
                    1000.0 / avg_mspt
                } else {
                    20.0
                };
                let color = if avg_mspt > 40.0 {
                    NamedColor::Red
                } else if avg_mspt > 25.0 {
                    NamedColor::Gold
                } else {
                    NamedColor::Green
                };
                let tps_text = TextComponent::text("TPS")
                    .color_named(NamedColor::Gray)
                    .add_child(TextComponent::text(": ").color_named(NamedColor::White))
                    .add_child(TextComponent::text(format!("{tps:.2}")).color_named(color));
                let mspt_text = TextComponent::text("MSPT")
                    .color_named(NamedColor::Gray)
                    .add_child(TextComponent::text(": ").color_named(NamedColor::White))
                    .add_child(TextComponent::text(format!("{avg_mspt:.2}")).color_named(color));
                player
                    .client
                    .enqueue_packet(
                        &CTabList::default().with_footer(
                            tps_text
                                .add_child(TextComponent::text(" "))
                                .add_child(mspt_text),
                        ),
                    )
                    .await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }
}

#[plugin_method]
async fn on_load(&mut self, server: &Context) -> Result<(), String> {
    pumpkin::init_log!();

    log::info!("Hello, Pumpkin!");

    server
        .register_event(Arc::new(MyJoinHandler), EventPriority::Lowest, false)
        .await;

    Ok(())
}

#[plugin_impl]
pub struct MyPlugin {}

impl MyPlugin {
    pub fn new() -> Self {
        MyPlugin {}
    }
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self::new()
    }
}
