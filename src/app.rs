use eframe::egui;
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use winreg::enums::*;
use winreg::RegKey;

use crate::language::Language;

const ENV_KEY: &str = "Environment";

#[derive(Deserialize, Serialize, Clone, Debug)]
struct VersionEntry {
    path: String,
    alias: String,
}

// Strukturen für den Path Cleaner
#[derive(Clone, Debug, PartialEq)]
enum IssueType {
    Missing,   // Ordner existiert nicht
    Duplicate, // Ordner steht doppelt drin
}

#[derive(Clone, Debug)]
struct CleanerEntry {
    path: String,
    issue: IssueType,
    selected: bool,
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

    #[serde(skip)]
    editing_index: Option<usize>,
    #[serde(skip)]
    edit_name_buffer: String,
    #[serde(skip)]
    edit_path_buffer: String,

    // Cleaner Status
    #[serde(skip)]
    show_cleaner_window: bool,
    #[serde(skip)]
    cleaner_issues: Vec<CleanerEntry>,

    // NEU: Such-Status
    #[serde(skip)]
    search_query: String,
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
            show_cleaner_window: false,
            cleaner_issues: Vec::new(),
            search_query: String::new(),
        }
    }
}

impl VersionSwitcherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::configure_styles(&cc.egui_ctx);
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    fn configure_styles(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        let orange = egui::Color32::from_rgb(255, 140, 0);
        let dark_gray = egui::Color32::from_rgb(30, 30, 30);

        visuals.widgets.noninteractive.bg_fill = dark_gray;
        visuals.selection.bg_fill = orange;
        visuals.selection.stroke = egui::Stroke::new(1.0, orange);

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

    fn set_path_var(&mut self, new_path: String, active_alias: Option<&str>) -> Result<(), String> {
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

                // Notification nur zeigen, wenn wir explizit eine Version gewechselt haben
                if let Some(alias) = active_alias {
                    Notification::new()
                        .summary(self.app_language.notify_title())
                        .body(&self.app_language.notify_body(alias))
                        .appname("Version Switcher")
                        .show()
                        .ok();
                }
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

        match self.set_path_var(new_path_str, Some(target_alias)) {
            Ok(_) => self.status_message = self.app_language.status_activated(target_path),
            Err(e) => self.status_message = self.app_language.status_error(&e),
        }
    }

    // Export Funktion
    fn export_config(&mut self) {
        if let Some(path) = rfd::FileDialog::new().set_file_name("version_switcher_config.json").save_file() {
            match File::create(path) {
                Ok(file) => {
                    let writer = BufWriter::new(file);
                    if let Err(e) = serde_json::to_writer_pretty(writer, &self.languages) {
                        self.status_message = self.app_language.status_export_err(&e.to_string());
                    } else {
                        self.status_message = self.app_language.status_export_ok().to_string();
                    }
                },
                Err(e) => self.status_message = self.app_language.status_export_err(&e.to_string()),
            }
        }
    }

    // Import Funktion
    fn import_config(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            match File::open(path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    match serde_json::from_reader(reader) {
                        Ok(languages) => {
                            self.languages = languages;
                            if !self.languages.contains_key(&self.selected_group) {
                                if let Some(first_key) = self.languages.keys().next() {
                                    self.selected_group = first_key.clone();
                                } else {
                                    self.selected_group = "General".to_owned();
                                }
                            }
                            self.status_message = self.app_language.status_import_ok().to_string();
                        },
                        Err(e) => self.status_message = self.app_language.status_import_err(&e.to_string()),
                    }
                },
                Err(e) => self.status_message = self.app_language.status_import_err(&e.to_string()),
            }
        }
    }

    // Path Cleaner - Scannen
    fn scan_path_issues(&mut self) {
        let current_path = self.get_current_path_var();
        let parts: Vec<&str> = current_path.split(';').filter(|s| !s.is_empty()).collect();

        let mut entries = Vec::new();
        let mut seen = HashSet::new();

        for p in parts {
            let p_string = p.to_string();
            let mut issue = None;

            // Check Duplicate (Case Insensitive für Windows)
            if seen.contains(&p_string.to_lowercase()) {
                issue = Some(IssueType::Duplicate);
            } else {
                seen.insert(p_string.to_lowercase());
                // Check Existence
                if !Path::new(p).exists() {
                    issue = Some(IssueType::Missing);
                }
            }

            if let Some(iss) = issue {
                entries.push(CleanerEntry {
                    path: p_string,
                    issue: iss,
                    selected: true,
                });
            }
        }
        self.cleaner_issues = entries;
    }

    // Path Cleaner - Aufräumen
    fn clean_selected_paths(&mut self) {
        let to_remove_missing: HashSet<String> = self.cleaner_issues.iter()
            .filter(|e| e.selected && e.issue == IssueType::Missing)
            .map(|e| e.path.to_lowercase())
            .collect();

        let to_deduplicate: HashSet<String> = self.cleaner_issues.iter()
            .filter(|e| e.selected && e.issue == IssueType::Duplicate)
            .map(|e| e.path.to_lowercase())
            .collect();

        if to_remove_missing.is_empty() && to_deduplicate.is_empty() {
            return;
        }

        let current_path = self.get_current_path_var();
        let parts: Vec<&str> = current_path.split(';').filter(|s| !s.is_empty()).collect();
        let mut new_parts = Vec::new();
        let mut seen = HashSet::new();
        let mut removed_count = 0;

        for p in parts {
            let p_lower = p.to_lowercase();
            if to_remove_missing.contains(&p_lower) {
                removed_count += 1;
                continue;
            }
            if to_deduplicate.contains(&p_lower) {
                if seen.contains(&p_lower) {
                    removed_count += 1;
                    continue;
                }
            }
            seen.insert(p_lower);
            new_parts.push(p);
        }

        let new_path_str = new_parts.join(";");

        if let Ok(_) = self.set_path_var(new_path_str, None) {
            self.status_message = self.app_language.status_cleaned(removed_count);
            self.scan_path_issues();
        } else {
            self.status_message = "Error writing Path".to_owned();
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
                    assert!(class == egui::ViewportClass::Immediate, "This egui backend doesn't support multiple viewports");

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading(lang.window_cleaner_title());
                        if ui.button(lang.btn_scan()).clicked() {
                            self.scan_path_issues();
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
                                self.clean_selected_paths();
                            }
                        }
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_cleaner_window = false;
                    }
                }
            );
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

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Import / Export
                    if ui.button("📥").on_hover_text(self.app_language.tooltip_import()).clicked() {
                        self.import_config();
                    }
                    if ui.button("📤").on_hover_text(self.app_language.tooltip_export()).clicked() {
                        self.export_config();
                    }

                    // Cleaner Button
                    ui.add_space(5.0);
                    if ui.button("🧹").on_hover_text(self.app_language.tooltip_cleaner()).clicked() {
                        self.show_cleaner_window = !self.show_cleaner_window;
                        if self.show_cleaner_window {
                            self.scan_path_issues();
                        }
                    }
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

            // --- HEADER LISTE & SUCHE ---
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(self.app_language.header_available()).heading());

                // Flexible Space
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                    // FIX: Explizites Left-to-Right Layout innerhalb des Right-to-Left Headers.
                    // Das sorgt dafür, dass:
                    // 1. Die Sucheingabe ZUERST kommt (Index 0) -> Fokus bleibt stabil
                    // 2. Der Button DANACH kommt (Index 1) -> Visuell rechts daneben
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let hint = match self.app_language {
                            Language::English => "Search...",
                            Language::German => "Suchen...",
                        };

                        // Suchfeld
                        ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text(hint).desired_width(150.0));

                        // Clear-Button (erscheint rechts vom Feld)
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

            // Suchbegriff für Filter vorbereiten
            let query = self.search_query.to_lowercase();
            let has_filter = !query.is_empty();

            if let Some(versions) = self.languages.get_mut(&self.selected_group) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let len = versions.len();

                    for (idx, entry) in versions.iter_mut().enumerate() {
                        // --- LIVE-FILTER ---
                        if has_filter {
                            let matches_alias = entry.alias.to_lowercase().contains(&query);
                            let matches_path = entry.path.to_lowercase().contains(&query);
                            if !matches_alias && !matches_path {
                                continue; // Eintrag überspringen wenn er nicht passt
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

                                    // Sortier Pfeile nur zeigen wenn KEIN Filter aktiv ist (sonst verwirrend)
                                    if !has_filter {
                                        ui.vertical(|ui| {
                                            if idx > 0 {
                                                if ui.small_button("⬆").on_hover_text(lang.tooltip_move_up()).clicked() { move_up = Some(idx); }
                                            }
                                            if idx < len - 1 {
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