use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use winreg::enums::*;
use winreg::RegKey;
use notify_rust::Notification;

use crate::types::{CleanerEntry, IssueType, VersionEntry};

const ENV_KEY: &str = "Environment";

// --- REGISTRY FUNKTIONEN ---

pub fn get_current_path_var() -> String {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey(ENV_KEY).unwrap_or_else(|_| hkcu.create_subkey(ENV_KEY).unwrap().0);
    env.get_value("Path").unwrap_or_default()
}

pub fn set_path_var(new_path: String) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = match hkcu.open_subkey_with_flags(ENV_KEY, KEY_WRITE) {
        Ok(key) => key,
        Err(e) => return Err(format!("Registry Error: {}", e)),
    };

    match env.set_value("Path", &new_path) {
        Ok(_) => {
            // Windows benachrichtigen (Broadcast)
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
            Ok(())
        },
        Err(e) => Err(format!("Write Error: {}", e)),
    }
}

pub fn send_notification(title: &str, body: &str) {
    Notification::new()
        .summary(title)
        .body(body)
        .appname("Version Switcher")
        .show()
        .ok();
}

// --- IMPORT / EXPORT FUNKTIONEN ---

pub fn export_to_file(data: &HashMap<String, Vec<VersionEntry>>) -> Result<(), String> {
    if let Some(path) = rfd::FileDialog::new().set_file_name("version_switcher_config.json").save_file() {
        let file = File::create(path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, data).map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err("Cancelled".to_string())
}

pub fn import_from_file() -> Result<HashMap<String, Vec<VersionEntry>>, String> {
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader).map_err(|e| e.to_string())?;
        return Ok(data);
    }
    Err("Cancelled".to_string())
}

// --- CLEANER FUNKTIONEN ---

pub fn scan_for_issues(current_path: &str) -> Vec<CleanerEntry> {
    let parts: Vec<&str> = current_path.split(';').filter(|s| !s.is_empty()).collect();
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    for p in parts {
        let p_string = p.to_string();
        let mut issue = None;

        if seen.contains(&p_string.to_lowercase()) {
            issue = Some(IssueType::Duplicate);
        } else {
            seen.insert(p_string.to_lowercase());
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
    entries
}

pub fn perform_cleanup(current_path: &str, issues: &[CleanerEntry]) -> (String, usize) {
    let to_remove_missing: HashSet<String> = issues.iter()
        .filter(|e| e.selected && e.issue == IssueType::Missing)
        .map(|e| e.path.to_lowercase())
        .collect();

    let to_deduplicate: HashSet<String> = issues.iter()
        .filter(|e| e.selected && e.issue == IssueType::Duplicate)
        .map(|e| e.path.to_lowercase())
        .collect();

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

    (new_parts.join(";"), removed_count)
}