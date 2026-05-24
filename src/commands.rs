use pumpkin_plugin_api::{
    Server,
    command_wit::{
        Arg, ArgumentType, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType,
    },
    commands::{Command, CommandHandler},
    text::{NamedColor, TextComponent},
};

use crate::{
    config,
    sampler::{self, Averages},
    toggle::{self, ToggleField},
};

/// Top-level permission required to even see `/tabtps`. Sub-permissions are
/// checked inside each handler so a player without `tabtps.command.reload`
/// still discovers `/tabtps toggle`.
pub const PERM_USE: &str = "tabtps.command.use";
pub const PERM_RELOAD: &str = "tabtps.command.reload";
pub const PERM_TOGGLE: &str = "tabtps.command.toggle";
pub const PERM_TICKINFO: &str = "tabtps.command.tickinfo";
pub const PERM_PING: &str = "tabtps.command.ping";
pub const PERM_PINGALL: &str = "tabtps.command.pingall";

/// Builds the full `/tabtps` command tree. The caller is responsible for
/// registering the [`PERM_USE`] / [`PERM_RELOAD`] / [`PERM_TOGGLE`] permission
/// nodes first via `context.register_permission`.
pub fn build_tabtps() -> Command {
    let toggle = CommandNode::literal("toggle");
    toggle.then(CommandNode::literal("tab").execute(ToggleHandler(ToggleField::Tab)));
    toggle.then(CommandNode::literal("actionbar").execute(ToggleHandler(ToggleField::Actionbar)));
    toggle.then(CommandNode::literal("bossbar").execute(ToggleHandler(ToggleField::Bossbar)));

    let cmd = Command::new(&["tabtps".into()], "TabTPS plugin commands");
    cmd.then(toggle);
    cmd.then(CommandNode::literal("reload").execute(ReloadHandler));
    cmd
}

/// `/tickinfo` (alias `/mspt`) — dumps the current TPS / MSPT rolling
/// averages to the sender.
pub fn build_tickinfo() -> Command {
    Command::new(
        &["tickinfo".into(), "mspt".into()],
        "Show TPS and MSPT rolling averages (1m / 5m / 15m)",
    )
    .execute(TickinfoHandler)
}

/// `/ping` — own ping for the sender; `/ping <name>` for another online
/// player.
pub fn build_ping() -> Command {
    let cmd = Command::new(
        &["ping".into()],
        "Show your own ping, or a named player's ping",
    )
    .execute(PingSelfHandler);
    let name_arg = CommandNode::argument("name", &ArgumentType::String(StringType::SingleWord))
        .execute(PingOtherHandler);
    cmd.then(name_arg);
    cmd
}

/// `/pingall` — comma-separated list of every online player's ping.
pub fn build_pingall() -> Command {
    Command::new(&["pingall".into()], "Show every online player's ping").execute(PingAllHandler)
}

struct ReloadHandler;

impl CommandHandler for ReloadHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        if !sender.has_permission(&server, PERM_RELOAD) {
            return Err(CommandError::PermissionDenied);
        }
        match config::reload() {
            Ok(_) => {
                sender.send_message(coloured(
                    "TabTPS config reloaded from tabtps.toml",
                    NamedColor::Green,
                ));
                Ok(0)
            }
            Err(err) => Err(CommandError::CommandFailed(coloured(
                &format!("Could not reload TabTPS config: {err}"),
                NamedColor::Red,
            ))),
        }
    }
}

struct ToggleHandler(ToggleField);

impl CommandHandler for ToggleHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        if !sender.has_permission(&server, PERM_TOGGLE) {
            return Err(CommandError::PermissionDenied);
        }
        let Some(player) = sender.as_player() else {
            return Err(CommandError::CommandFailed(coloured(
                "/tabtps toggle is player-only",
                NamedColor::Red,
            )));
        };
        let id = player.get_id();
        let key = (id.high, id.low);
        let now_on = toggle::flip(key, self.0);
        let state = if now_on { "enabled" } else { "disabled" };
        let colour = if now_on {
            NamedColor::Green
        } else {
            NamedColor::Gray
        };
        sender.send_message(coloured(
            &format!("{} display: {state}", self.0.label()),
            colour,
        ));
        Ok(0)
    }
}

fn coloured(message: &str, colour: NamedColor) -> TextComponent {
    let text = TextComponent::text(message);
    text.color_named(colour);
    text
}

struct TickinfoHandler;

impl CommandHandler for TickinfoHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        if !sender.has_permission(&server, PERM_TICKINFO) {
            return Err(CommandError::PermissionDenied);
        }
        let tps = sampler::tps_averages();
        let mspt = sampler::mspt_averages();
        sender.send_message(coloured(
            &format!("TPS:  {}", format_averages(tps)),
            NamedColor::Gold,
        ));
        sender.send_message(coloured(
            &format!("MSPT: {}", format_averages(mspt)),
            NamedColor::Gold,
        ));
        Ok(0)
    }
}

struct PingSelfHandler;

impl CommandHandler for PingSelfHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        if !sender.has_permission(&server, PERM_PING) {
            return Err(CommandError::PermissionDenied);
        }
        let Some(player) = sender.as_player() else {
            return Err(CommandError::CommandFailed(coloured(
                "/ping with no argument is player-only — try /ping <name>",
                NamedColor::Red,
            )));
        };
        sender.send_message(coloured(
            &format!("Your ping: {}ms", player.get_ping()),
            NamedColor::Gold,
        ));
        Ok(0)
    }
}

struct PingOtherHandler;

impl CommandHandler for PingOtherHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        if !sender.has_permission(&server, PERM_PING) {
            return Err(CommandError::PermissionDenied);
        }
        let name = match args.get_value("name") {
            Arg::Simple(s) => s,
            _ => {
                return Err(CommandError::CommandFailed(coloured(
                    "Expected a player name",
                    NamedColor::Red,
                )));
            }
        };
        match server.get_player_by_name(&name) {
            Some(player) => {
                sender.send_message(coloured(
                    &format!("{}'s ping: {}ms", player.get_name(), player.get_ping()),
                    NamedColor::Gold,
                ));
                Ok(0)
            }
            None => Err(CommandError::CommandFailed(coloured(
                &format!("No online player named {name}"),
                NamedColor::Red,
            ))),
        }
    }
}

struct PingAllHandler;

impl CommandHandler for PingAllHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        if !sender.has_permission(&server, PERM_PINGALL) {
            return Err(CommandError::PermissionDenied);
        }
        let mut entries: Vec<(String, u32)> = server
            .get_all_players()
            .into_iter()
            .map(|p| (p.get_name(), p.get_ping()))
            .collect();
        if entries.is_empty() {
            sender.send_message(coloured("No players online", NamedColor::Gray));
            return Ok(0);
        }
        entries.sort_by_key(|(name, _)| name.to_lowercase());
        let body = entries
            .iter()
            .map(|(name, ping)| format!("{name}: {ping}ms"))
            .collect::<Vec<_>>()
            .join(", ");
        sender.send_message(coloured(&body, NamedColor::Gold));
        Ok(0)
    }
}

fn format_averages(avg: Averages) -> String {
    if avg.is_nan() {
        return "—, —, —".to_string();
    }
    format!(
        "{:.2}, {:.2}, {:.2}",
        avg.one_min, avg.five_min, avg.fifteen_min,
    )
}
