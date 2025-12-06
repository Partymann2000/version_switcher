// Verstecke das Konsolenfenster im Release-Modus unter Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod language;

use app::VersionSwitcherApp;

fn main() -> eframe::Result<()> {
    // Fenstergröße leicht angepasst für die neuen Elemente
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