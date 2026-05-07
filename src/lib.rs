mod join_handler;

use pumpkin_plugin_api::{Context, Plugin, PluginMetadata, events::EventPriority};

use crate::join_handler::TabtpsJoinHandler;

struct TabtpsPlugin;

impl Plugin for TabtpsPlugin {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: env!("CARGO_PKG_AUTHORS")
                .split(',')
                .map(String::from)
                .collect(),
            description: env!("CARGO_PKG_DESCRIPTION").to_string(),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        tracing::info!("Hello, TabTPS!");

        context.register_event_handler(TabtpsJoinHandler, EventPriority::Normal, true)?;

        Ok(())
    }

    fn on_unload(&mut self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        tracing::info!("Unloading TabTPS plugin");

        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(TabtpsPlugin);
