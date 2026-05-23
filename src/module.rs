use pumpkin_plugin_api::{
    Server,
    text::{NamedColor, TextComponent},
};

/// A single labelled value rendered into the tab list footer (or header).
///
/// Implementors decide what data to sample from [`Server`], how to format it,
/// and which color to use. Modules are composed by the renderer with a
/// fixed separator between them.
pub trait Module {
    fn render(&self, server: &Server) -> TextComponent;
}

pub struct TpsModule;
pub struct MsptModule;

impl Module for TpsModule {
    fn render(&self, server: &Server) -> TextComponent {
        let tps = server.get_tps();
        let mspt = server.get_mspt();
        labeled_value("TPS", &format!("{tps:.2}"), color_for_mspt(mspt))
    }
}

impl Module for MsptModule {
    fn render(&self, server: &Server) -> TextComponent {
        let mspt = server.get_mspt();
        labeled_value("MSPT", &format!("{mspt:.2}"), color_for_mspt(mspt))
    }
}

fn color_for_mspt(mspt: f64) -> NamedColor {
    match mspt {
        ..25.0 => NamedColor::Green,
        ..40.0 => NamedColor::Gold,
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
