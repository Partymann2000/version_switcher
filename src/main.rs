// Verstecke das Konsolenfenster im Release-Modus unter Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Hier registrieren wir die neuen Dateien als Module
mod app;
mod language;

// Wir nutzen die App aus dem 'app' Modul
use app::VersionSwitcherApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([600.0, 650.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Version Switcher",
        options,
        Box::new(|cc| Ok(Box::new(VersionSwitcherApp::new(cc)))),
    )
}