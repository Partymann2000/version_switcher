use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VersionEntry {
    pub path: String,
    pub alias: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IssueType {
    Missing,
    Duplicate,
}

#[derive(Clone, Debug)]
pub struct CleanerEntry {
    pub path: String,
    pub issue: IssueType,
    pub selected: bool,
}

// NEU: Eintrag für den Verlauf
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HistoryEntry {
    pub time: String,    // z.B. "14:30:05"
    pub message: String, // z.B. "Activated Python 3.11"
}