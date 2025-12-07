use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// Wir importieren unsere neuen Module
use crate::language::Language;
use crate::types::{VersionEntry, CleanerEntry, IssueType};
use crate::logic;
use crate::style;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct VersionSwitcherApp {
    languages: HashMap<String, Vec<VersionEntry>>,
    selected_group: String,
    app_language: Language,
    accent_color: [u8; 3],

    #[serde(skip)]
    new_group_name: String,
    #[serde(skip)]
    new_path_input: String,
    #[serde(skip)]
    new_alias_input: String,
    #[serde(skip)]
    status_message: String,

    #[serde(skip)]
    editing_index: Option<usize>,
    #[serde(skip)]
    edit_name_buffer: String,
    #[serde(skip)]
    edit_path_buffer: String,

    #[serde(skip)]
    show_cleaner_window: bool,
    #[serde(skip)]
    cleaner_issues: Vec<CleanerEntry>,

    #[serde(skip)]
    search_query: String,
}

impl Default for VersionSwitcherApp {
    fn default() -> Self {
        Self {
            languages: HashMap::new(),
            selected_group: "General".to_owned(),
            app_language: Language::German,
            accent_color: [255, 140, 0],
            new_group_name: String::new(),
            new_path_input: String::new(),
            new_alias_input: String::new(),
            status_message: "Bereit.".to_owned(),
            editing_index: None,
            edit_name_buffer: String::new(),
            edit_path_buffer: String::new(),
            show_cleaner_window: false,
            cleaner_issues: Vec::new(),
            search_query: String::new(),
        }
    }
}

impl VersionSwitcherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app: VersionSwitcherApp = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        // Design anwenden
        style::apply_style(&cc.egui_ctx, app.accent_color);
        app
    }

    // --- HELPER WRAPPER ---

    fn switch_version(&mut self, target_path: &str, target_alias: &str) {
        let current_path_str = logic::get_current_path_var();
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

        // Logik aufrufen
        match logic::set_path_var(new_path_str) {
            Ok(_) => {
                logic::send_notification(
                    self.app_language.notify_title(),
                    &self.app_language.notify_body(target_alias)
                );
                self.status_message = self.app_language.status_activated(target_path);
            },
            Err(e) => self.status_message = self.app_language.status_error(&e),
        }
    }

    fn run_export(&mut self) {
        match logic::export_to_file(&self.languages) {
            Ok(_) => self.status_message = self.app_language.status_export_ok().to_string(),
            Err(e) if e == "Cancelled" => {},
            Err(e) => self.status_message = self.app_language.status_export_err(&e),
        }
    }

    fn run_import(&mut self) {
        match logic::import_from_file() {
            Ok(data) => {
                self.languages = data;
                // Auswahl korrigieren falls nötig
                if !self.languages.contains_key(&self.selected_group) {
                    if let Some(key) = self.languages.keys().next() {
                        self.selected_group = key.clone();
                    } else {
                        self.selected_group = "General".to_owned();
                    }
                }
                self.status_message = self.app_language.status_import_ok().to_string();
            },
            Err(e) if e == "Cancelled" => {},
            Err(e) => self.status_message = self.app_language.status_import_err(&e),
        }
    }

    fn run_cleaner(&mut self) {
        let current = logic::get_current_path_var();
        let (new_path, count) = logic::perform_cleanup(&current, &self.cleaner_issues);

        if count > 0 {
            if let Ok(_) = logic::set_path_var(new_path) {
                self.status_message = self.app_language.status_cleaned(count);
                self.cleaner_issues = logic::scan_for_issues(&logic::get_current_path_var());
            } else {
                self.status_message = "Error writing Path".to_owned();
            }
        }
    }
}

impl eframe::App for VersionSwitcherApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drag & Drop
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

        // Cleaner Fenster
        if self.show_cleaner_window {
            let lang = self.app_language;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("cleaner_window"),
                egui::ViewportBuilder::default()
                    .with_title(lang.window_cleaner_title())
                    .with_inner_size([500.0, 400.0]),
                |ctx, class| {
                    assert!(class == egui::ViewportClass::Immediate, "Backend error");
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading(lang.window_cleaner_title());
                        if ui.button(lang.btn_scan()).clicked() {
                            self.cleaner_issues = logic::scan_for_issues(&logic::get_current_path_var());
                        }
                        ui.separator();

                        if self.cleaner_issues.is_empty() {
                            ui.label(lang.label_no_issues());
                        } else {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for entry in &mut self.cleaner_issues {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut entry.selected, "");
                                        match entry.issue {
                                            IssueType::Missing => {
                                                ui.colored_label(egui::Color32::RED, format!("[{}]", lang.issue_missing()));
                                            }
                                            IssueType::Duplicate => {
                                                ui.colored_label(egui::Color32::YELLOW, format!("[{}]", lang.issue_duplicate()));
                                            }
                                        }
                                        ui.label(&entry.path);
                                    });
                                }
                            });
                            ui.separator();
                            if ui.button(lang.btn_clean_selected()).clicked() {
                                self.run_cleaner();
                            }
                        }
                    });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_cleaner_window = false;
                    }
                }
            );
        }

        let current_sys_path_str = logic::get_current_path_var();
        let current_sys_paths: Vec<String> = current_sys_path_str.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                let accent = egui::Color32::from_rgb(self.accent_color[0], self.accent_color[1], self.accent_color[2]);
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

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Import/Export
                    if ui.button("📥").on_hover_text(self.app_language.tooltip_import()).clicked() {
                        self.run_import();
                    }
                    if ui.button("📤").on_hover_text(self.app_language.tooltip_export()).clicked() {
                        self.run_export();
                    }

                    // Cleaner
                    ui.add_space(5.0);
                    if ui.button("🧹").on_hover_text(self.app_language.tooltip_cleaner()).clicked() {
                        self.show_cleaner_window = !self.show_cleaner_window;
                        if self.show_cleaner_window {
                            self.cleaner_issues = logic::scan_for_issues(&logic::get_current_path_var());
                        }
                    }

                    // Farbe
                    ui.add_space(5.0);
                    if egui::color_picker::color_edit_button_srgb(ui, &mut self.accent_color).changed() {
                        style::apply_style(ctx, self.accent_color);
                    }
                    ui.label("🎨").on_hover_text(self.app_language.tooltip_accent_color());
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

            // Neuer Eintrag
            ui.group(|ui| {
                let accent = egui::Color32::from_rgb(self.accent_color[0], self.accent_color[1], self.accent_color[2]);
                ui.label(egui::RichText::new(self.app_language.header_add_version(&self.selected_group)).strong().color(accent));

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

            // Header Liste & Suche
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(self.app_language.header_available()).heading());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let hint = match self.app_language {
                            Language::English => "Search...",
                            Language::German => "Suchen...",
                        };
                        // ID für Fokus-Stabilität
                        ui.push_id("search_query_input", |ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text(hint).desired_width(150.0));
                        });
                        if !self.search_query.is_empty() {
                            if ui.button("❌").clicked() {
                                self.search_query.clear();
                            }
                        }
                    });
                    ui.label("🔍");
                });
            });

            // Liste & Aktionen
            let mut move_up = None;
            let mut move_down = None;
            let mut delete_index = None;
            let mut start_edit = None;
            let mut save_edit = None;
            let mut cancel_edit = false;
            let mut activate_version = None;

            let lang = self.app_language;
            let query = self.search_query.to_lowercase();
            let has_filter = !query.is_empty();

            if let Some(versions) = self.languages.get_mut(&self.selected_group) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // FIX: Länge vorher speichern, um Mutable Borrow Konflikte zu vermeiden
                    let versions_len = versions.len();

                    for (idx, entry) in versions.iter_mut().enumerate() {
                        if has_filter {
                            let matches_alias = entry.alias.to_lowercase().contains(&query);
                            let matches_path = entry.path.to_lowercase().contains(&query);
                            if !matches_alias && !matches_path {
                                continue;
                            }
                        }

                        ui.group(|ui| {
                            if self.editing_index == Some(idx) {
                                // Editier Modus
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    ui.text_edit_singleline(&mut self.edit_name_buffer);
                                    ui.label("Pfad:");
                                    ui.text_edit_singleline(&mut self.edit_path_buffer);

                                    if ui.button("💾").on_hover_text(lang.tooltip_save()).clicked() {
                                        save_edit = Some(idx);
                                    }
                                    if ui.button("❌").on_hover_text(lang.tooltip_cancel()).clicked() {
                                        cancel_edit = true;
                                    }
                                });
                            } else {
                                // Anzeige Modus
                                ui.horizontal(|ui| {
                                    let is_active = current_sys_paths.iter().any(|p| p.eq_ignore_ascii_case(&entry.path));

                                    if is_active { ui.label("🟢"); } else { ui.label("⚪"); }

                                    if !has_filter {
                                        ui.vertical(|ui| {
                                            if idx > 0 {
                                                if ui.small_button("⬆").on_hover_text(lang.tooltip_move_up()).clicked() { move_up = Some(idx); }
                                            }
                                            // FIX: Hier versions_len nutzen statt versions.len()
                                            if idx < versions_len - 1 {
                                                if ui.small_button("⬇").on_hover_text(lang.tooltip_move_down()).clicked() { move_down = Some(idx); }
                                            }
                                        });
                                    }

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
                                        if ui.button("🗑").on_hover_text(lang.tooltip_delete()).clicked() { delete_index = Some(idx); }
                                        if ui.button("✏").on_hover_text(lang.tooltip_edit()).clicked() { start_edit = Some((idx, entry.alias.clone(), entry.path.clone())); }

                                        let btn_text = if is_active { lang.btn_is_active() } else { lang.btn_activate() };
                                        let btn = egui::Button::new(btn_text).selected(is_active);
                                        if ui.add_enabled(!is_active, btn).clicked() {
                                            activate_version = Some((entry.path.clone(), entry.alias.clone()));
                                        }
                                    });
                                });
                            }
                        });
                    }
                });

                // Aktionen
                if let Some(idx) = move_up { versions.swap(idx, idx - 1); }
                if let Some(idx) = move_down { versions.swap(idx, idx + 1); }

                if let Some((idx, name, path)) = start_edit {
                    self.editing_index = Some(idx);
                    self.edit_name_buffer = name;
                    self.edit_path_buffer = path;
                }

                if let Some(idx) = save_edit {
                    if let Some(entry) = versions.get_mut(idx) {
                        entry.alias = self.edit_name_buffer.clone();
                        entry.path = self.edit_path_buffer.clone();
                    }
                    self.editing_index = None;
                }

                if cancel_edit { self.editing_index = None; }

                if let Some(idx) = delete_index {
                    versions.remove(idx);
                    if self.editing_index == Some(idx) { self.editing_index = None; }
                }
            }

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