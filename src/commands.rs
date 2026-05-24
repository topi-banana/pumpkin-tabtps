use pumpkin_plugin_api::{
    Server,
    command_wit::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::{Command, CommandHandler},
    text::{NamedColor, TextComponent},
};

use crate::{
    config,
    toggle::{self, ToggleField},
};

/// Top-level permission required to even see `/tabtps`. Sub-permissions are
/// checked inside each handler so a player without `tabtps.command.reload`
/// still discovers `/tabtps toggle`.
pub const PERM_USE: &str = "tabtps.command.use";
pub const PERM_RELOAD: &str = "tabtps.command.reload";
pub const PERM_TOGGLE: &str = "tabtps.command.toggle";

/// Builds the full `/tabtps` command tree. The caller is responsible for
/// registering the [`PERM_USE`] / [`PERM_RELOAD`] / [`PERM_TOGGLE`] permission
/// nodes first via `context.register_permission`.
pub fn build() -> Command {
    let toggle = CommandNode::literal("toggle");
    toggle.then(CommandNode::literal("tab").execute(ToggleHandler(ToggleField::Tab)));
    toggle.then(CommandNode::literal("actionbar").execute(ToggleHandler(ToggleField::Actionbar)));
    toggle.then(CommandNode::literal("bossbar").execute(ToggleHandler(ToggleField::Bossbar)));

    let cmd = Command::new(&["tabtps".into()], "TabTPS plugin commands");
    cmd.then(toggle);
    cmd.then(CommandNode::literal("reload").execute(ReloadHandler));
    cmd
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
