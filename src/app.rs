use eframe::egui;
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

use crate::language::Language;

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
    app_language: Language,

    // UI-Status Variablen
    #[serde(skip)]
    new_group_name: String,
    #[serde(skip)]
    new_path_input: String,
    #[serde(skip)]
    new_alias_input: String,
    #[serde(skip)]
    status_message: String,

    // NEU: Status für das Editieren eines Eintrags
    // Wir merken uns den Index des Elements, das gerade bearbeitet wird
    #[serde(skip)]
    editing_index: Option<usize>,
    #[serde(skip)]
    edit_name_buffer: String,
    #[serde(skip)]
    edit_path_buffer: String,
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
            editing_index: None,
            edit_name_buffer: String::new(),
            edit_path_buffer: String::new(),
        }
    }
}

impl VersionSwitcherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // --- 1. CUSTOM STYLING (Dark Mode + Orange) ---
        Self::configure_styles(&cc.egui_ctx);

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    // Funktion für das Design
    fn configure_styles(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // Farben anpassen (Orange Akzente)
        let orange = egui::Color32::from_rgb(255, 140, 0); // Dark Orange
        let dark_gray = egui::Color32::from_rgb(30, 30, 30);

        visuals.widgets.noninteractive.bg_fill = dark_gray;
        visuals.selection.bg_fill = orange;
        visuals.selection.stroke = egui::Stroke::new(1.0, orange);

        // Abrundungen für modernere Optik
        visuals.window_rounding = egui::Rounding::same(8.0);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
        visuals.widgets.active.rounding = egui::Rounding::same(4.0);
        visuals.widgets.open.rounding = egui::Rounding::same(4.0);

        ctx.set_visuals(visuals);
    }

    fn get_current_path_var(&self) -> String {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey(ENV_KEY).unwrap_or_else(|_| hkcu.create_subkey(ENV_KEY).unwrap().0);
        env.get_value("Path").unwrap_or_default()
    }

    fn set_path_var(&mut self, new_path: String, active_alias: &str, _active_path: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = match hkcu.open_subkey_with_flags(ENV_KEY, KEY_WRITE) {
            Ok(key) => key,
            Err(e) => return Err(format!("Registry Error: {}", e)),
        };

        match env.set_value("Path", &new_path) {
            Ok(_) => {
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

    fn switch_version(&mut self, target_path: &str, target_alias: &str) {
        let current_path_str = self.get_current_path_var();
        let mut parts: Vec<String> = current_path_str.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if let Some(versions) = self.languages.get(&self.selected_group) {
            for v in versions {
                parts.retain(|p| !p.eq_ignore_ascii_case(&v.path));
            }
        }

        parts.insert(0, target_path.to_string());
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
        // --- 2. DRAG & DROP LOGIK ---
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            for file in dropped_files {
                if let Some(path) = file.path {
                    if path.is_dir() {
                        self.new_path_input = path.display().to_string();
                        if self.new_alias_input.is_empty() {
                            if let Some(folder_name) = path.file_name() {
                                self.new_alias_input = folder_name.to_string_lossy().to_string();
                            }
                        }
                    }
                }
            }
        }

        let current_sys_path_str = self.get_current_path_var();
        let current_sys_paths: Vec<String> = current_sys_path_str.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Windows Version Switcher").color(egui::Color32::WHITE));
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

            // Gruppen Auswahl
            ui.horizontal(|ui| {
                ui.label(self.app_language.label_group_select());
                egui::ComboBox::from_id_salt("group_select")
                    .selected_text(egui::RichText::new(&self.selected_group).strong())
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for lang in self.languages.keys() {
                            ui.selectable_value(&mut self.selected_group, lang.clone(), lang);
                        }
                    });

                ui.add_space(10.0);
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

            // Neuer Eintrag Hinzufügen
            ui.group(|ui| {
                ui.label(egui::RichText::new(self.app_language.header_add_version(&self.selected_group)).strong().color(egui::Color32::from_rgb(255, 140, 0)));

                let mut add_clicked = false;
                ui.horizontal(|ui| {
                    ui.label(self.app_language.label_name());
                    ui.add(egui::TextEdit::singleline(&mut self.new_alias_input).desired_width(80.0).hint_text(self.app_language.hint_name()));

                    ui.label(self.app_language.label_path());
                    let path_field = ui.add(egui::TextEdit::singleline(&mut self.new_path_input).desired_width(200.0).hint_text(self.app_language.hint_path()));

                    if ui.button("📂").on_hover_text(self.app_language.tooltip_folder()).clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.new_path_input = path.display().to_string();
                            if self.new_alias_input.is_empty() {
                                if let Some(folder_name) = path.file_name() {
                                    self.new_alias_input = folder_name.to_string_lossy().to_string();
                                }
                            }
                        }
                    }

                    if !self.new_path_input.is_empty() {
                        if Path::new(&self.new_path_input).is_dir() {
                            ui.label("✅").on_hover_text(self.app_language.status_path_ok());
                        } else {
                            ui.label("❌").on_hover_text(self.app_language.status_path_missing());
                        }
                    }

                    if path_field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        add_clicked = true;
                    }
                });

                ui.add_space(5.0);
                if ui.add_sized([ui.available_width(), 25.0], egui::Button::new(format!("➕ {}", self.app_language.btn_add()))).clicked() {
                    add_clicked = true;
                }

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
            ui.label(egui::RichText::new(self.app_language.header_available()).heading());

            // --- 3. & 4. SORTIEREN & EDITIEREN LOGIK ---
            let mut move_up = None;
            let mut move_down = None;
            let mut delete_index = None;
            let mut start_edit = None;
            let mut save_edit = None;
            let mut cancel_edit = false;
            // FIX: Variable um den Switch-Wunsch zu speichern, statt direkt auszuführen
            let mut activate_version = None;

            // FIX: Lokale Variable für Language, um self-Borrowing in der Schleife zu vermeiden
            let lang = self.app_language;

            if let Some(versions) = self.languages.get_mut(&self.selected_group) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let len = versions.len();

                    for (idx, entry) in versions.iter_mut().enumerate() {
                        ui.group(|ui| {
                            // IST DIESER EINTRAG GERADE IM EDITIER-MODUS?
                            if self.editing_index == Some(idx) {
                                // --- EDITIER MODUS ---
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    ui.text_edit_singleline(&mut self.edit_name_buffer);
                                    ui.label("Pfad:");
                                    ui.text_edit_singleline(&mut self.edit_path_buffer);

                                    // Speichern
                                    if ui.button("💾").on_hover_text(lang.tooltip_save()).clicked() {
                                        save_edit = Some(idx);
                                    }
                                    // Abbrechen
                                    if ui.button("❌").on_hover_text(lang.tooltip_cancel()).clicked() {
                                        cancel_edit = true;
                                    }
                                });
                            } else {
                                // --- ANZEIGE MODUS ---
                                ui.horizontal(|ui| {
                                    let is_active = current_sys_paths.iter().any(|p| p.eq_ignore_ascii_case(&entry.path));

                                    if is_active { ui.label("🟢"); } else { ui.label("⚪"); }

                                    // Sortier Pfeile
                                    ui.vertical(|ui| {
                                        if idx > 0 {
                                            if ui.small_button("⬆").on_hover_text(lang.tooltip_move_up()).clicked() { move_up = Some(idx); }
                                        }
                                        if idx < len - 1 {
                                            if ui.small_button("⬇").on_hover_text(lang.tooltip_move_down()).clicked() { move_down = Some(idx); }
                                        }
                                    });

                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(&entry.alias).strong().size(16.0));
                                        let path_exists = Path::new(&entry.path).is_dir();
                                        let path_text = egui::RichText::new(&entry.path).small().weak();
                                        if !path_exists {
                                            ui.horizontal(|ui| {
                                                ui.label(path_text.color(egui::Color32::RED));
                                                ui.label("⚠️").on_hover_text(lang.tooltip_missing_folder());
                                            });
                                        } else {
                                            ui.label(path_text);
                                        }
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // Löschen
                                        if ui.button("🗑").on_hover_text(lang.tooltip_delete()).clicked() { delete_index = Some(idx); }

                                        // Editieren Starten
                                        if ui.button("✏").on_hover_text(lang.tooltip_edit()).clicked() { start_edit = Some((idx, entry.alias.clone(), entry.path.clone())); }

                                        // Aktivieren
                                        let btn_text = if is_active { lang.btn_is_active() } else { lang.btn_activate() };
                                        let btn = egui::Button::new(btn_text).selected(is_active);
                                        if ui.add_enabled(!is_active, btn).clicked() {
                                            // FIX: NICHT self.switch_version aufrufen, sondern merken!
                                            activate_version = Some((entry.path.clone(), entry.alias.clone()));
                                        }
                                    });
                                });
                            }
                        });
                    }
                });

                // --- AKTIONEN DURCHFÜHREN (Nach der Schleife) ---
                // 1. Sortieren
                if let Some(idx) = move_up { versions.swap(idx, idx - 1); }
                if let Some(idx) = move_down { versions.swap(idx, idx + 1); }

                // 2. Editieren Starten
                if let Some((idx, name, path)) = start_edit {
                    self.editing_index = Some(idx);
                    self.edit_name_buffer = name;
                    self.edit_path_buffer = path;
                }

                // 3. Editieren Speichern
                if let Some(idx) = save_edit {
                    if let Some(entry) = versions.get_mut(idx) {
                        entry.alias = self.edit_name_buffer.clone();
                        entry.path = self.edit_path_buffer.clone();
                    }
                    self.editing_index = None;
                }

                // 4. Editieren Abbrechen
                if cancel_edit {
                    self.editing_index = None;
                }

                // 5. Löschen
                if let Some(idx) = delete_index {
                    versions.remove(idx);
                    if self.editing_index == Some(idx) { self.editing_index = None; }
                }
            } // Hier endet der Borrow von self.languages (versions)

            // FIX: Hier führen wir das Umschalten aus, da "versions" nicht mehr ausgeliehen ist
            if let Some((path, alias)) = activate_version {
                self.switch_version(&path, &alias);
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