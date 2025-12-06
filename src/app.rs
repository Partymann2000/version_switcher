use eframe::egui;
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

// Wir importieren das Language Enum aus unserer neuen Datei
use crate::language::Language;

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
    selected_group: String,

    // Die App-Sprache
    app_language: Language,

    // UI-Status Variablen (werden nicht gespeichert)
    #[serde(skip)]
    new_group_name: String,
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
            selected_group: "General".to_owned(),
            app_language: Language::German,
            new_group_name: String::new(),
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

    // Schreibt den PATH und benachrichtigt Windows
    fn set_path_var(&mut self, new_path: String, active_alias: &str, _active_path: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = match hkcu.open_subkey_with_flags(ENV_KEY, KEY_WRITE) {
            Ok(key) => key,
            Err(e) => return Err(format!("Registry Error: {}", e)),
        };

        match env.set_value("Path", &new_path) {
            Ok(_) => {
                // 1. Technische Nachricht an Windows
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

                // 2. Visuelle Nachricht (in der gewählten Sprache)
                Notification::new()
                    .summary(self.app_language.notify_title())
                    .body(&self.app_language.notify_body(active_alias))
                    .appname("Version Switcher")
                    .show()
                    .ok();

                Ok(())
            },
            Err(e) => Err(format!("Write Error: {}", e)),
        }
    }

    // Die Logik zum Austauschen
    fn switch_version(&mut self, target_path: &str, target_alias: &str) {
        let current_path_str = self.get_current_path_var();
        let mut parts: Vec<String> = current_path_str.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        // 1. Alte Versionen dieser Gruppe entfernen
        if let Some(versions) = self.languages.get(&self.selected_group) {
            for v in versions {
                parts.retain(|p| !p.eq_ignore_ascii_case(&v.path));
            }
        }

        // 2. Neuen Pfad vorne einfügen
        parts.insert(0, target_path.to_string());

        // 3. Zusammenbauen und Schreiben
        let new_path_str = parts.join(";");

        match self.set_path_var(new_path_str, target_alias, target_path) {
            Ok(_) => self.status_message = self.app_language.status_activated(target_path),
            Err(e) => self.status_message = self.app_language.status_error(&e),
        }
    }
}

impl eframe::App for VersionSwitcherApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_sys_path_str = self.get_current_path_var();

        // Splitte PATH für exakten Abgleich
        let current_sys_paths: Vec<String> = current_sys_path_str.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header mit Sprachauswahl
            ui.horizontal(|ui| {
                ui.heading("Windows Version Switcher");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("app_lang_select")
                        .width(100.0)
                        .selected_text(match self.app_language {
                            Language::English => "🇺🇸 English",
                            Language::German => "🇩🇪 Deutsch",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.app_language, Language::English, "🇺🇸 English");
                            ui.selectable_value(&mut self.app_language, Language::German, "🇩🇪 Deutsch");
                        });
                    ui.label(self.app_language.label_app_language());
                });
            });

            ui.add_space(10.0);

            // --- GRUPPEN AUSWÄHLEN (z.B. Python) ---
            ui.horizontal(|ui| {
                ui.label(self.app_language.label_group_select());
                egui::ComboBox::from_id_salt("group_select")
                    .selected_text(&self.selected_group)
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for lang in self.languages.keys() {
                            ui.selectable_value(&mut self.selected_group, lang.clone(), lang);
                        }
                    });

                ui.add_space(10.0);

                // Neue Gruppe hinzufügen
                ui.text_edit_singleline(&mut self.new_group_name)
                    .on_hover_text(self.app_language.tooltip_new_group());

                if ui.button(self.app_language.btn_new_group()).clicked() {
                    if !self.new_group_name.is_empty() {
                        self.languages.entry(self.new_group_name.clone()).or_insert_with(Vec::new);
                        self.selected_group = self.new_group_name.clone();
                        self.new_group_name.clear();
                    }
                }
            });

            ui.separator();

            // --- NEUE VERSION HINZUFÜGEN ---
            ui.group(|ui| {
                ui.label(egui::RichText::new(self.app_language.header_add_version(&self.selected_group)).strong());

                let mut add_clicked = false;

                ui.horizontal(|ui| {
                    // NAME INPUT
                    ui.label(self.app_language.label_name());
                    ui.add(egui::TextEdit::singleline(&mut self.new_alias_input)
                        .desired_width(80.0)
                        .hint_text(self.app_language.hint_name()));

                    // PFAD INPUT & BUTTONS
                    ui.label(self.app_language.label_path());
                    let path_field = ui.add(egui::TextEdit::singleline(&mut self.new_path_input)
                        .desired_width(200.0)
                        .hint_text(self.app_language.hint_path()));

                    // -- Ordner-Auswahl Dialog --
                    if ui.button("📂").on_hover_text(self.app_language.tooltip_folder()).clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.new_path_input = path.display().to_string();
                        }
                    }

                    // -- Validierung --
                    if !self.new_path_input.is_empty() {
                        let path_exists = Path::new(&self.new_path_input).is_dir();
                        if path_exists {
                            ui.label("✅").on_hover_text(self.app_language.status_path_ok());
                        } else {
                            ui.label("❌").on_hover_text(self.app_language.status_path_missing());
                        }
                    }

                    // Enter im Pfad-Feld
                    if path_field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        add_clicked = true;
                    }
                });

                ui.add_space(5.0);

                // Button
                let btn = egui::Button::new(format!("➕ {}", self.app_language.btn_add()));
                if ui.add_enabled(!self.new_path_input.is_empty(), btn).clicked() {
                    add_clicked = true;
                }

                // Logik zum Hinzufügen
                if add_clicked && !self.new_path_input.is_empty() {
                    if let Some(versions) = self.languages.get_mut(&self.selected_group) {
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

            // --- LISTE DER VERSIONEN ---
            ui.label(egui::RichText::new(self.app_language.header_available()).heading());

            let mut versions_clone = Vec::new();
            if let Some(versions) = self.languages.get(&self.selected_group) {
                versions_clone = versions.clone();
            }

            let mut delete_index = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, entry) in versions_clone.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Exakter Pfad-Check
                            let is_active = current_sys_paths.iter()
                                .any(|p| p.eq_ignore_ascii_case(&entry.path));

                            // STATUS ICON
                            if is_active {
                                ui.label("🟢");
                            } else {
                                ui.label("⚪");
                            }

                            // INFO TEXT
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&entry.alias).strong());
                                // Prüfen ob Pfad existiert
                                let path_exists = Path::new(&entry.path).is_dir();
                                let path_text = egui::RichText::new(&entry.path).small().weak();
                                if !path_exists {
                                    ui.horizontal(|ui| {
                                        ui.label(path_text.color(egui::Color32::RED));
                                        ui.label("⚠️").on_hover_text(self.app_language.tooltip_missing_folder());
                                    });
                                } else {
                                    ui.label(path_text);
                                }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // LÖSCHEN
                                if ui.button("🗑").on_hover_text(self.app_language.tooltip_delete()).clicked() {
                                    delete_index = Some(idx);
                                }

                                // AKTIVIEREN
                                let btn_text = if is_active { self.app_language.btn_is_active() } else { self.app_language.btn_activate() };
                                let btn = egui::Button::new(btn_text).selected(is_active);

                                if ui.add_enabled(!is_active, btn).clicked() {
                                    self.switch_version(&entry.path, &entry.alias);
                                }
                            });
                        });
                    });
                }
            });

            if let Some(idx) = delete_index {
                if let Some(versions) = self.languages.get_mut(&self.selected_group) {
                    versions.remove(idx);
                }
            }

            ui.add_space(10.0);
            ui.separator();
            ui.label(format!("Status: {}",
                             if self.status_message == "Bereit." || self.status_message == "Ready." {
                                 self.app_language.status_ready().to_string()
                             } else {
                                 self.status_message.clone()
                             }
            ));

            ui.collapsing("System PATH (Debug)", |ui| {
                ui.monospace(current_sys_path_str);
            });
        });
    }
}