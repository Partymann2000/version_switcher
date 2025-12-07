use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Clone, Copy, Debug)]
pub enum Language {
    English,
    German,
}

impl Language {
    pub fn label_app_language(&self) -> &str {
        match self {
            Language::English => "App Language:",
            Language::German => "App Sprache:",
        }
    }

    pub fn label_group_select(&self) -> &str {
        match self {
            Language::English => "Group:",
            Language::German => "Gruppe:",
        }
    }

    pub fn tooltip_new_group(&self) -> &str {
        match self {
            Language::English => "Name for new group (e.g. Python)",
            Language::German => "Name für neue Gruppe (z.B. Python)",
        }
    }

    pub fn btn_new_group(&self) -> &str {
        match self {
            Language::English => "New Group",
            Language::German => "Neue Gruppe",
        }
    }

    // NEU: Tooltip für Gruppe löschen
    pub fn tooltip_delete_group(&self) -> &str {
        match self {
            Language::English => "Delete current group",
            Language::German => "Aktuelle Gruppe löschen",
        }
    }

    pub fn header_add_version(&self, group: &str) -> String {
        match self {
            Language::English => format!("Add new {} version", group),
            Language::German => format!("Neue {} Version hinzufügen", group),
        }
    }

    pub fn label_name(&self) -> &str {
        match self {
            Language::English => "Name:",
            Language::German => "Name:",
        }
    }

    pub fn label_path(&self) -> &str {
        match self {
            Language::English => "Path:",
            Language::German => "Pfad:",
        }
    }

    pub fn hint_name(&self) -> &str {
        "v1.0"
    }

    pub fn hint_path(&self) -> &str {
        match self {
            Language::English => "C:\\Program Files\\... (or drag & drop folder)",
            Language::German => "C:\\Programme\\... (oder Ordner hierher ziehen)",
        }
    }

    pub fn tooltip_folder(&self) -> &str {
        match self {
            Language::English => "Select folder...",
            Language::German => "Ordner auswählen...",
        }
    }

    pub fn status_path_ok(&self) -> &str {
        match self {
            Language::English => "Path exists",
            Language::German => "Pfad existiert",
        }
    }

    pub fn status_path_missing(&self) -> &str {
        match self {
            Language::English => "Path not found",
            Language::German => "Pfad nicht gefunden",
        }
    }

    pub fn btn_add(&self) -> &str {
        match self {
            Language::English => "Add",
            Language::German => "Hinzufügen",
        }
    }

    pub fn header_available(&self) -> &str {
        match self {
            Language::English => "Available Versions",
            Language::German => "Verfügbare Versionen",
        }
    }

    pub fn tooltip_delete(&self) -> &str {
        match self {
            Language::English => "Remove",
            Language::German => "Entfernen",
        }
    }

    pub fn tooltip_edit(&self) -> &str {
        match self {
            Language::English => "Edit",
            Language::German => "Bearbeiten",
        }
    }

    pub fn tooltip_save(&self) -> &str {
        match self {
            Language::English => "Save changes",
            Language::German => "Änderungen speichern",
        }
    }

    pub fn tooltip_cancel(&self) -> &str {
        match self {
            Language::English => "Cancel",
            Language::German => "Abbrechen",
        }
    }

    pub fn tooltip_move_up(&self) -> &str {
        match self {
            Language::English => "Move up",
            Language::German => "Nach oben",
        }
    }

    pub fn tooltip_move_down(&self) -> &str {
        match self {
            Language::English => "Move down",
            Language::German => "Nach unten",
        }
    }

    pub fn tooltip_missing_folder(&self) -> &str {
        match self {
            Language::English => "Folder not found on disk!",
            Language::German => "Ordner auf Festplatte nicht gefunden!",
        }
    }

    pub fn btn_activate(&self) -> &str {
        match self {
            Language::English => "Activate",
            Language::German => "Aktivieren",
        }
    }

    pub fn btn_is_active(&self) -> &str {
        match self {
            Language::English => "Active",
            Language::German => "Ist Aktiv",
        }
    }

    pub fn notify_title(&self) -> &str {
        match self {
            Language::English => "Environment Switched",
            Language::German => "Version gewechselt",
        }
    }

    pub fn notify_body(&self, alias: &str) -> String {
        match self {
            Language::English => format!("{} is now active.", alias),
            Language::German => format!("{} ist jetzt aktiv.", alias),
        }
    }

    pub fn status_ready(&self) -> &str {
        match self {
            Language::English => "Ready.",
            Language::German => "Bereit.",
        }
    }

    pub fn status_activated(&self, path: &str) -> String {
        match self {
            Language::English => format!("Activated: {}", path),
            Language::German => format!("Aktiviert: {}", path),
        }
    }

    pub fn status_error(&self, err: &str) -> String {
        format!("Error/Fehler: {}", err)
    }

    pub fn tooltip_import(&self) -> &str {
        match self {
            Language::English => "Import configuration",
            Language::German => "Konfiguration importieren",
        }
    }

    pub fn tooltip_export(&self) -> &str {
        match self {
            Language::English => "Export configuration",
            Language::German => "Konfiguration exportieren",
        }
    }

    pub fn status_import_ok(&self) -> &str {
        match self {
            Language::English => "Configuration imported successfully.",
            Language::German => "Konfiguration erfolgreich importiert.",
        }
    }

    pub fn status_export_ok(&self) -> &str {
        match self {
            Language::English => "Configuration exported successfully.",
            Language::German => "Konfiguration erfolgreich exportiert.",
        }
    }

    pub fn status_import_err(&self, err: &str) -> String {
        match self {
            Language::English => format!("Import Error: {}", err),
            Language::German => format!("Import Fehler: {}", err),
        }
    }

    pub fn status_export_err(&self, err: &str) -> String {
        match self {
            Language::English => format!("Export Error: {}", err),
            Language::German => format!("Export Fehler: {}", err),
        }
    }

    pub fn tooltip_cleaner(&self) -> &str {
        match self {
            Language::English => "Path Cleaner",
            Language::German => "Pfad-Bereinigung",
        }
    }

    pub fn window_cleaner_title(&self) -> &str {
        match self {
            Language::English => "System Path Cleaner",
            Language::German => "System-Pfad Bereinigung",
        }
    }

    pub fn btn_scan(&self) -> &str {
        match self {
            Language::English => "Scan Path Now",
            Language::German => "Pfad jetzt scannen",
        }
    }

    pub fn label_no_issues(&self) -> &str {
        match self {
            Language::English => "No issues found.",
            Language::German => "Keine Probleme gefunden.",
        }
    }

    pub fn btn_clean_selected(&self) -> &str {
        match self {
            Language::English => "Clean Selected",
            Language::German => "Ausgewählte bereinigen",
        }
    }

    pub fn status_cleaned(&self, count: usize) -> String {
        match self {
            Language::English => format!("Removed {} entries.", count),
            Language::German => format!("{} Einträge entfernt.", count),
        }
    }

    pub fn issue_missing(&self) -> &str {
        match self {
            Language::English => "Missing",
            Language::German => "Fehlt",
        }
    }

    pub fn issue_duplicate(&self) -> &str {
        match self {
            Language::English => "Duplicate",
            Language::German => "Duplikat",
        }
    }

    pub fn tooltip_accent_color(&self) -> &str {
        match self {
            Language::English => "Change accent color",
            Language::German => "Akzentfarbe ändern",
        }
    }

    pub fn tooltip_history(&self) -> &str {
        match self {
            Language::English => "Activity History",
            Language::German => "Aktivitäts-Verlauf",
        }
    }

    pub fn window_history_title(&self) -> &str {
        match self {
            Language::English => "Activity History",
            Language::German => "Aktivitäts-Verlauf",
        }
    }

    pub fn btn_clear_history(&self) -> &str {
        match self {
            Language::English => "Clear History",
            Language::German => "Verlauf leeren",
        }
    }

    pub fn label_no_history(&self) -> &str {
        match self {
            Language::English => "No activity recorded yet.",
            Language::German => "Noch keine Aktivitäten aufgezeichnet.",
        }
    }
}