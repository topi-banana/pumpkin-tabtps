use pumpkin_plugin_api::{
    Server,
    player::Player,
    text::{NamedColor, TextComponent},
};

/// A single labelled value rendered into the tab list footer (or header).
///
/// Implementors decide what data to sample from [`Server`] and the viewing
/// [`Player`], how to format it, and which color to use. Modules are composed
/// by the renderer with a fixed separator between them.
pub trait Module {
    fn render(&self, server: &Server, player: &Player) -> TextComponent;
}

pub struct TpsModule;
pub struct MsptModule;
pub struct PlayerCountModule;
pub struct PingModule;

/// Render `modules` into a single [`TextComponent`], separating successive
/// entries with a space. Returns an empty component when `modules` is empty,
/// which lets callers leave a tab list slot blank without special-casing.
pub fn compose(modules: &[&dyn Module], server: &Server, player: &Player) -> TextComponent {
    let mut parts = modules.iter();
    let Some(first) = parts.next() else {
        return TextComponent::text("");
    };
    let component = first.render(server, player);
    for module in parts {
        component.add_child(TextComponent::text(" "));
        component.add_child(module.render(server, player));
    }
    component
}

impl Module for TpsModule {
    fn render(&self, server: &Server, _player: &Player) -> TextComponent {
        let tps = server.get_tps();
        let mspt = server.get_mspt();
        labeled_value("TPS", &format!("{tps:.2}"), color_for_mspt(mspt))
    }
}

impl Module for MsptModule {
    fn render(&self, server: &Server, _player: &Player) -> TextComponent {
        let mspt = server.get_mspt();
        labeled_value("MSPT", &format!("{mspt:.2}"), color_for_mspt(mspt))
    }
}

impl Module for PlayerCountModule {
    fn render(&self, server: &Server, _player: &Player) -> TextComponent {
        let current = server.get_player_count();
        let max = server.get_max_players();
        labeled_value("Players", &format!("{current}/{max}"), NamedColor::White)
    }
}

impl Module for PingModule {
    fn render(&self, _server: &Server, player: &Player) -> TextComponent {
        let ping = player.get_ping();
        labeled_value("Ping", &format!("{ping}ms"), color_for_ping(ping))
    }
}

fn color_for_mspt(mspt: f64) -> NamedColor {
    match mspt {
        ..25.0 => NamedColor::Green,
        ..40.0 => NamedColor::Gold,
        _ => NamedColor::Red,
    }
}

fn color_for_ping(ping: u32) -> NamedColor {
    match ping {
        ..100 => NamedColor::Green,
        100..200 => NamedColor::Gold,
        _ => NamedColor::Red,
    }
}

fn labeled_value(name: &'static str, value: &str, color: NamedColor) -> TextComponent {
    let label = TextComponent::text(name);
    label.color_named(NamedColor::Gray);
    label.add_child({
        let sep = TextComponent::text(": ");
        sep.color_named(NamedColor::White);
        sep
    });
    label.add_child({
        let v = TextComponent::text(value);
        v.color_named(color);
        v
    });
    label
}
