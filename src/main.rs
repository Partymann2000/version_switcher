// Verstecke das Konsolenfenster im Release-Modus unter Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path; // Wichtig für die Pfad-Prüfung
use winreg::enums::*;
use winreg::RegKey;

// Registry-Schlüssel
const ENV_KEY: &str = "Environment";

#[derive(Deserialize, Serialize, Clone, Debug)]
struct VersionEntry {
    path: String,
    alias: String,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct VersionSwitcherApp {
    languages: HashMap<String, Vec<VersionEntry>>,
    selected_language: String,

    // UI-Status Variablen (werden nicht gespeichert)
    #[serde(skip)]
    new_lang_name: String,
    #[serde(skip)]
    new_path_input: String,
    #[serde(skip)]
    new_alias_input: String,
    #[serde(skip)]
    status_message: String,
}

impl Default for VersionSwitcherApp {
    fn default() -> Self {
        Self {
            languages: HashMap::new(),
            selected_language: "General".to_owned(),
            new_lang_name: String::new(),
            new_path_input: String::new(),
            new_alias_input: String::new(),
            status_message: "Bereit.".to_owned(),
        }
    }
}

impl VersionSwitcherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    // Holt den aktuellen PATH aus der Registry
    fn get_current_path_var(&self) -> String {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey(ENV_KEY).unwrap_or_else(|_| hkcu.create_subkey(ENV_KEY).unwrap().0);
        env.get_value("Path").unwrap_or_default()
    }

    // Schreibt den PATH und benachrichtigt Windows (Technisch + Visuell)
    fn set_path_var(&mut self, new_path: String, active_alias: &str, active_path: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = match hkcu.open_subkey_with_flags(ENV_KEY, KEY_WRITE) {
            Ok(key) => key,
            Err(e) => return Err(format!("Registry Fehler: {}", e)),
        };

        match env.set_value("Path", &new_path) {
            Ok(_) => {
                // 1. Technische Nachricht an Windows (damit Programme es merken)
                use std::ptr;
                #[cfg(windows)]
                unsafe {
                    use winapi::um::winuser::{SendMessageTimeoutA, HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG};
                    let lp_param = std::ffi::CString::new("Environment").unwrap();
                    SendMessageTimeoutA(
                        HWND_BROADCAST, WM_SETTINGCHANGE, 0, lp_param.as_ptr() as isize,
                        SMTO_ABORTIFHUNG, 5000, ptr::null_mut(),
                    );
                }

                // 2. Visuelle Nachricht an den Benutzer (Popup)
                Notification::new()
                    .summary("Version gewechselt")
                    .body(&format!("{} ist jetzt aktiv.\nPfad: {}", active_alias, active_path))
                    .appname("Version Switcher")
                    .show()
                    .ok(); // Fehler ignorieren, falls Benachrichtigungen deaktiviert sind

                Ok(())
            },
            Err(e) => Err(format!("Schreibfehler: {}", e)),
        }
    }

    // Die Logik zum Austauschen
    fn switch_version(&mut self, target_path: &str, target_alias: &str) {
        let current_path_str = self.get_current_path_var();
        let mut parts: Vec<String> = current_path_str.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        // 1. Alte Versionen dieser Sprache entfernen
        if let Some(versions) = self.languages.get(&self.selected_language) {
            for v in versions {
                // Entferne Pfad, wenn er bereits existiert (Case-Insensitive)
                parts.retain(|p| !p.eq_ignore_ascii_case(&v.path));
            }
        }

        // 2. Neuen Pfad vorne einfügen
        parts.insert(0, target_path.to_string());

        // 3. Zusammenbauen und Schreiben
        let new_path_str = parts.join(";");

        match self.set_path_var(new_path_str, target_alias, target_path) {
            Ok(_) => self.status_message = format!("Aktiviert: {}", target_path),
            Err(e) => self.status_message = format!("Fehler: {}", e),
        }
    }
}

impl eframe::App for VersionSwitcherApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_sys_path_str = self.get_current_path_var();

        // BUG FIX: Wir zerlegen den PATH String in einzelne Elemente.
        // So können wir exakt prüfen (Exact Match) statt nur "beinhaltet" (Contains).
        // Das verhindert, dass "...\Python\3.14" und "...\Python\3.14\Scripts" gleichzeitig grün leuchten.
        let current_sys_paths: Vec<String> = current_sys_path_str.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Windows Version Switcher");
            ui.add_space(5.0);

            // --- SPRACHE AUSWÄHLEN ---
            ui.horizontal(|ui| {
                ui.label("Sprache:");
                egui::ComboBox::from_id_salt("lang_select")
                    .selected_text(&self.selected_language)
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for lang in self.languages.keys() {
                            ui.selectable_value(&mut self.selected_language, lang.clone(), lang);
                        }
                    });

                ui.add_space(10.0);

                // Neue Sprache hinzufügen
                ui.text_edit_singleline(&mut self.new_lang_name).on_hover_text("Name für neue Gruppe");
                if ui.button("Neue Gruppe").clicked() {
                    if !self.new_lang_name.is_empty() {
                        self.languages.entry(self.new_lang_name.clone()).or_insert_with(Vec::new);
                        self.selected_language = self.new_lang_name.clone();
                        self.new_lang_name.clear();
                    }
                }
            });

            ui.separator();

            // --- NEUE VERSION HINZUFÜGEN ---
            ui.group(|ui| {
                ui.label(egui::RichText::new(format!("Neue {} Version hinzufügen", self.selected_language)).strong());

                let mut add_clicked = false;

                ui.horizontal(|ui| {
                    // NAME INPUT
                    ui.label("Name:");
                    ui.add(egui::TextEdit::singleline(&mut self.new_alias_input).desired_width(80.0).hint_text("v1.0"));

                    // PFAD INPUT & BUTTONS
                    ui.label("Pfad:");
                    let path_field = ui.add(egui::TextEdit::singleline(&mut self.new_path_input).desired_width(200.0).hint_text("C:\\Program\\v1"));

                    // -- Ordner-Auswahl Dialog --
                    if ui.button("📂").on_hover_text("Ordner auswählen...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.new_path_input = path.display().to_string();
                        }
                    }

                    // -- Validierung (Existenz prüfen) --
                    if !self.new_path_input.is_empty() {
                        let path_exists = Path::new(&self.new_path_input).is_dir();
                        if path_exists {
                            ui.label("✅").on_hover_text("Pfad existiert");
                        } else {
                            ui.label("❌").on_hover_text("Pfad nicht gefunden / existiert nicht");
                        }
                    }

                    // Enter im Pfad-Feld löst Hinzufügen aus
                    if path_field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        add_clicked = true;
                    }
                });

                ui.add_space(5.0);

                // Button deaktivieren wenn Pfad leer ist
                let btn = egui::Button::new("➕ Hinzufügen");
                if ui.add_enabled(!self.new_path_input.is_empty(), btn).clicked() {
                    add_clicked = true;
                }

                // Logik zum Hinzufügen
                if add_clicked && !self.new_path_input.is_empty() {
                    if let Some(versions) = self.languages.get_mut(&self.selected_language) {
                        versions.push(VersionEntry {
                            path: self.new_path_input.clone(),
                            alias: if self.new_alias_input.is_empty() { "Unbenannt".to_string() } else { self.new_alias_input.clone() },
                        });
                        self.new_path_input.clear();
                        self.new_alias_input.clear();
                    }
                }
            });

            ui.add_space(10.0);

            // --- LISTE DER VERSIONEN (ANZEIGEN & SCHALTEN) ---
            ui.label(egui::RichText::new("Verfügbare Versionen").heading());

            let mut versions_clone = Vec::new();
            if let Some(versions) = self.languages.get(&self.selected_language) {
                versions_clone = versions.clone();
            }

            let mut delete_index = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, entry) in versions_clone.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // BUG FIX LOGIC: Prüfe ob dieser Pfad GENAU so in der Liste der Systempfade vorkommt
                            let is_active = current_sys_paths.iter()
                                .any(|p| p.eq_ignore_ascii_case(&entry.path));

                            // STATUS ICON
                            if is_active {
                                ui.label("🟢"); // Aktiv
                            } else {
                                ui.label("⚪"); // Inaktiv
                            }

                            // INFO TEXT
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&entry.alias).strong());
                                // Prüfen ob der gespeicherte Pfad überhaupt noch existiert (Validierung in der Liste)
                                let path_exists = Path::new(&entry.path).is_dir();
                                let path_text = egui::RichText::new(&entry.path).small().weak();
                                if !path_exists {
                                    ui.horizontal(|ui| {
                                        ui.label(path_text.color(egui::Color32::RED));
                                        ui.label("⚠️").on_hover_text("Ordner nicht gefunden!");
                                    });
                                } else {
                                    ui.label(path_text);
                                }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // LÖSCHEN BUTTON
                                if ui.button("🗑").on_hover_text("Entfernen").clicked() {
                                    delete_index = Some(idx);
                                }

                                // AKTIVIEREN BUTTON
                                let btn_text = if is_active { "Ist Aktiv" } else { "Aktivieren" };
                                let btn = egui::Button::new(btn_text).selected(is_active);

                                if ui.add_enabled(!is_active, btn).clicked() {
                                    self.switch_version(&entry.path, &entry.alias);
                                }
                            });
                        });
                    });
                }
            });

            // Tatsächliches Löschen aus der echten Liste durchführen
            if let Some(idx) = delete_index {
                if let Some(versions) = self.languages.get_mut(&self.selected_language) {
                    versions.remove(idx);
                }
            }

            ui.add_space(10.0);
            ui.separator();
            ui.label(format!("Status: {}", self.status_message));

            ui.collapsing("System PATH (Debug)", |ui| {
                ui.monospace(current_sys_path_str);
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([550.0, 650.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Version Switcher",
        options,
        Box::new(|cc| Ok(Box::new(VersionSwitcherApp::new(cc)))),
    )
}