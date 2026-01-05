use std::{sync::Arc, time::Duration};

use pumpkin::{
    net::ClientPlatform,
    plugin::{BoxFuture, EventHandler, player::player_join::PlayerJoinEvent},
    server::Server,
};
use pumpkin_data::packet::clientbound::PLAY_TAB_LIST;
use pumpkin_protocol::packet::Packet;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use serde::Serialize;

#[derive(Serialize)]
struct CTabList {
    header: TextComponent,
    footer: TextComponent,
}

impl Packet for CTabList {
    const PACKET_ID: i32 = PLAY_TAB_LIST;
}

pub struct TabtpsJoinHandler {
    runtime: tokio::runtime::Runtime,
}

impl TabtpsJoinHandler {
    pub fn new() -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }
}

impl EventHandler<PlayerJoinEvent> for TabtpsJoinHandler {
    fn handle<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a PlayerJoinEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let player = Arc::clone(&event.player);
            let server = Arc::clone(server);

            let ClientPlatform::Java(_) = player.client else {
                return;
            };

            self.runtime.spawn(async move {
                while !player.client.closed() {
                    let nspts = server.get_tick_times_nanos_copy().await;
                    let avg_mspt = nspts.iter().copied().sum::<i64>() as f64 / 100.0 / 1_000_000.0;
                    let tps = if avg_mspt > 50.0 {
                        1000.0 / avg_mspt
                    } else {
                        20.0
                    };
                    let color = match avg_mspt {
                        ..25.0 => NamedColor::Green,
                        ..40.0 => NamedColor::Gold,
                        _ => NamedColor::Red,
                    };

                    let tps_text = gen_text_component("TPS", format!("{tps:.2}"), color);
                    let mspt_text = gen_text_component("MSPT", format!("{avg_mspt:.2}"), color);

                    player
                        .client
                        .enqueue_packet(&CTabList {
                            header: TextComponent::text(""),
                            footer: tps_text
                                .add_child(TextComponent::text(" "))
                                .add_child(mspt_text),
                        })
                        .await;

                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
        })
    }
}

fn gen_text_component(name: &'static str, value: String, color: NamedColor) -> TextComponent {
    TextComponent::text(name)
        .color_named(NamedColor::Gray)
        .add_child(TextComponent::text(": ").color_named(NamedColor::White))
        .add_child(TextComponent::text(value).color_named(color))
}
