mod commands;
mod config;
mod join_handler;
mod module;
mod sampler;
mod toggle;

use std::path::PathBuf;

use pumpkin_plugin_api::{
    Context, Plugin, PluginMetadata,
    events::EventPriority,
    permission::{Permission, PermissionDefault, PermissionLevel},
    permissions, scheduler,
};

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
            permissions: vec![
                permissions::FS_READ_DATA.to_string(),
                permissions::FS_WRITE_DATA.to_string(),
                permissions::SYS_INFO_RAM.to_string(),
            ],
        }
    }

    fn on_load(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        tracing::info!("Hello, TabTPS!");

        let data_folder = PathBuf::from(context.get_data_folder());
        config::init_data_folder(data_folder.clone());
        config::replace(config::load_from_disk(&data_folder));

        scheduler::schedule_repeating_task(20, 20, |server| {
            sampler::record(server.get_tps(), server.get_mspt());
        });

        register_permissions(&context)?;
        context.register_command(commands::build_tabtps(), commands::PERM_USE);
        context.register_command(commands::build_tickinfo(), commands::PERM_TICKINFO);
        context.register_command(commands::build_ping(), commands::PERM_PING);
        context.register_command(commands::build_pingall(), commands::PERM_PINGALL);
        context.register_command(commands::build_memory(), commands::PERM_MEMORY);

        context.register_event_handler(TabtpsJoinHandler, EventPriority::Normal, true)?;

        Ok(())
    }

    fn on_unload(&mut self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        tracing::info!("Unloading TabTPS plugin");

        Ok(())
    }
}

fn register_permissions(context: &Context) -> pumpkin_plugin_api::Result<()> {
    let perms = [
        Permission {
            node: commands::PERM_USE.into(),
            description: "Required to invoke any /tabtps subcommand".into(),
            default: PermissionDefault::Allow,
            children: vec![],
        },
        Permission {
            node: commands::PERM_RELOAD.into(),
            description: "Allows /tabtps reload (re-read tabtps.toml)".into(),
            default: PermissionDefault::Op(PermissionLevel::Four),
            children: vec![],
        },
        Permission {
            node: commands::PERM_TOGGLE.into(),
            description: "Allows /tabtps toggle <tab|actionbar|bossbar>".into(),
            default: PermissionDefault::Allow,
            children: vec![],
        },
        Permission {
            node: commands::PERM_TICKINFO.into(),
            description: "Allows /tickinfo (alias /mspt)".into(),
            default: PermissionDefault::Allow,
            children: vec![],
        },
        Permission {
            node: commands::PERM_PING.into(),
            description: "Allows /ping and /ping <name>".into(),
            default: PermissionDefault::Allow,
            children: vec![],
        },
        Permission {
            node: commands::PERM_PINGALL.into(),
            description: "Allows /pingall".into(),
            default: PermissionDefault::Allow,
            children: vec![],
        },
        Permission {
            node: commands::PERM_MEMORY.into(),
            description: "Allows /memory (/mem, /ram)".into(),
            default: PermissionDefault::Allow,
            children: vec![],
        },
    ];
    for perm in &perms {
        if let Err(err) = context.register_permission(perm) {
            tracing::warn!(node = %perm.node, error = %err, "Could not register permission");
        }
    }
    Ok(())
}

pumpkin_plugin_api::register_plugin!(TabtpsPlugin);
