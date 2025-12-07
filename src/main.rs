// Verstecke das Konsolenfenster im Release-Modus unter Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Module registrieren
mod types;
mod style;
mod logic;
mod language;
mod app;

use app::VersionSwitcherApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([650.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Version Switcher",
        options,
        Box::new(|cc| Ok(Box::new(VersionSwitcherApp::new(cc)))),
    )
}