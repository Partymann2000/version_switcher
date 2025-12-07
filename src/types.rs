use serde::{Deserialize, Serialize};

// Unsere Haupt-Datenstruktur für einen Eintrag
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VersionEntry {
    pub path: String,
    pub alias: String,
}

// Typen für den Path Cleaner
#[derive(Clone, Debug, PartialEq)]
pub enum IssueType {
    Missing,   // Ordner weg
    Duplicate, // Doppelter Eintrag
}

#[derive(Clone, Debug)]
pub struct CleanerEntry {
    pub path: String,
    pub issue: IssueType,
    pub selected: bool,
}