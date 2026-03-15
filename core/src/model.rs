use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "v001";

pub fn schema_version_string() -> String {
    SCHEMA_VERSION.to_owned()
}

pub const fn default_autosave_delay_ms() -> u64 {
    1_500
}

pub const fn default_max_snapshots_per_note() -> usize {
    200
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    #[default]
    System,
    Ja,
    En,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedLocale {
    Ja,
    En,
}

impl Locale {
    pub fn resolve(self) -> ResolvedLocale {
        match self {
            Self::Ja => ResolvedLocale::Ja,
            Self::En => ResolvedLocale::En,
            Self::System => Self::from_system_locale(),
        }
    }

    fn from_system_locale() -> ResolvedLocale {
        let candidates = [
            std::env::var("LC_ALL").ok(),
            std::env::var("LC_MESSAGES").ok(),
            std::env::var("LANG").ok(),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.to_ascii_lowercase().starts_with("ja") {
                return ResolvedLocale::Ja;
            }
        }

        ResolvedLocale::En
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryConfig {
    #[serde(default = "default_max_snapshots_per_note")]
    pub max_snapshots_per_note: usize,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_snapshots_per_note: default_max_snapshots_per_note(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "schema_version_string")]
    pub schema_version: String,
    #[serde(default)]
    pub locale: Locale,
    #[serde(default)]
    pub note_roots: Vec<PathBuf>,
    #[serde(default = "default_autosave_delay_ms")]
    pub autosave_delay_ms: u64,
    #[serde(default)]
    pub history: HistoryConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: schema_version_string(),
            locale: Locale::default(),
            note_roots: Vec::new(),
            autosave_delay_ms: default_autosave_delay_ms(),
            history: HistoryConfig::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for WindowPosition {
    fn default() -> Self {
        Self { x: 96.0, y: 96.0 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 360.0,
            height: 300.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowPlacement {
    #[serde(default)]
    pub position: WindowPosition,
    #[serde(default)]
    pub size: WindowSize,
    #[serde(default)]
    pub maximized: bool,
}

impl Default for WindowPlacement {
    fn default() -> Self {
        default_note_window()
    }
}

pub fn default_management_window() -> WindowPlacement {
    WindowPlacement {
        position: WindowPosition { x: 48.0, y: 48.0 },
        size: WindowSize {
            width: 420.0,
            height: 560.0,
        },
        maximized: false,
    }
}

pub fn default_note_window() -> WindowPlacement {
    WindowPlacement {
        position: WindowPosition::default(),
        size: WindowSize::default(),
        maximized: false,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenWindowState {
    pub root: PathBuf,
    pub note_id: String,
    #[serde(default)]
    pub position: WindowPosition,
    #[serde(default)]
    pub size: WindowSize,
    #[serde(default)]
    pub maximized: bool,
}

impl OpenWindowState {
    pub fn locator(&self) -> NoteLocator {
        NoteLocator {
            root: self.root.clone(),
            note_id: self.note_id.clone(),
        }
    }

    pub fn placement(&self) -> WindowPlacement {
        WindowPlacement {
            position: self.position.clone(),
            size: self.size.clone(),
            maximized: self.maximized,
        }
    }

    pub fn with_placement(locator: NoteLocator, placement: WindowPlacement) -> Self {
        Self {
            root: locator.root,
            note_id: locator.note_id,
            position: placement.position,
            size: placement.size,
            maximized: placement.maximized,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    #[serde(default = "schema_version_string")]
    pub schema_version: String,
    #[serde(default = "default_management_window")]
    pub management_window: WindowPlacement,
    #[serde(default)]
    pub open_windows: Vec<OpenWindowState>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            schema_version: schema_version_string(),
            management_window: default_management_window(),
            open_windows: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    #[serde(default = "schema_version_string")]
    pub schema_version: String,
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteLocator {
    pub root: PathBuf,
    pub note_id: String,
}

impl NoteLocator {
    pub fn new(root: PathBuf, note_id: impl Into<String>) -> Self {
        Self {
            root,
            note_id: note_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredNote {
    pub root: PathBuf,
    pub note: Note,
}

impl StoredNote {
    pub fn locator(&self) -> NoteLocator {
        NoteLocator {
            root: self.root.clone(),
            note_id: self.note.id.clone(),
        }
    }
}
