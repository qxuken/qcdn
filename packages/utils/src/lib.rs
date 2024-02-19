use std::io::IsTerminal;

pub use constants::*;

mod constants;

pub fn setup_color_eyre() -> color_eyre::Result<()> {
    color_eyre::config::HookBuilder::default()
        .theme(if !std::io::stderr().is_terminal() {
            // Don't attempt color
            color_eyre::config::Theme::new()
        } else {
            color_eyre::config::Theme::dark()
        })
        .install()
}
