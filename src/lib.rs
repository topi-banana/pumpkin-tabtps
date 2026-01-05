use std::sync::Arc;

use pumpkin::plugin::{Context, EventPriority, Plugin, PluginFuture, PluginMetadata};

mod join_handler;
use crate::join_handler::TabtpsJoinHandler;

pub struct TabtpsPlugin;

impl Plugin for TabtpsPlugin {
    fn on_load(&mut self, server: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            log::info!("Hello, Pumpkin!");

            server
                .register_event(
                    Arc::new(TabtpsJoinHandler::new()),
                    EventPriority::Lowest,
                    false,
                )
                .await;

            Ok(())
        })
    }

    fn on_unload(&mut self, _server: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

#[unsafe(no_mangle)]
pub fn plugin() -> Box<dyn Plugin> {
    Box::new(TabtpsPlugin)
}

#[unsafe(no_mangle)]
pub static METADATA: PluginMetadata = PluginMetadata {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    authors: env!("CARGO_PKG_AUTHORS"),
    description: env!("CARGO_PKG_DESCRIPTION"),
};
