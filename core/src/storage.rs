use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use directories::BaseDirs;
use tempfile::NamedTempFile;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use ulid::Ulid;

use crate::messages::MessageCatalog;
use crate::model::{
    Config, HistoryConfig, Note, NoteLocator, OpenWindowState, Session, StoredNote,
    WindowPlacement, default_note_window, schema_version_string,
};

const HISTORY_FILE_STAMP: &[time::format_description::BorrowedFormatItem<'static>] =
    format_description!("[year][month][day]T[hour][minute][second][subsecond digits:9]Z");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub config_file: PathBuf,
    pub session_file: PathBuf,
    pub activation_file: PathBuf,
    pub default_note_root: PathBuf,
}

impl AppPaths {
    pub fn new(config_dir: PathBuf, data_dir: PathBuf) -> Self {
        let default_note_root = data_dir.join("default-root");

        Self {
            config_file: config_dir.join("config.yml"),
            session_file: config_dir.join("session.yml"),
            activation_file: config_dir.join("activation.signal"),
            config_dir,
            data_dir,
            default_note_root,
        }
    }

    pub fn detect() -> Result<Self> {
        let base_dirs =
            BaseDirs::new().ok_or_else(|| anyhow!("failed to determine platform directories"))?;
        let app_folder = app_folder_name();
        let config_dir = base_dirs.config_dir().join(app_folder);
        let data_dir = base_dirs.data_local_dir().join(app_folder);

        Ok(Self::new(config_dir, data_dir))
    }
}

#[derive(Clone, Debug)]
pub struct AppBootstrap {
    pub paths: AppPaths,
    pub config: Config,
    pub session: Session,
    pub notes: Vec<StoredNote>,
    pub messages: MessageCatalog,
}

pub fn default_note_root(paths: &AppPaths) -> PathBuf {
    paths.default_note_root.clone()
}

pub fn load_or_bootstrap() -> Result<AppBootstrap> {
    let paths = AppPaths::detect()?;
    load_or_bootstrap_at(paths)
}

pub fn load_or_bootstrap_at(paths: AppPaths) -> Result<AppBootstrap> {
    fs::create_dir_all(&paths.config_dir)
        .with_context(|| format!("failed to create {}", paths.config_dir.display()))?;
    fs::create_dir_all(&paths.data_dir)
        .with_context(|| format!("failed to create {}", paths.data_dir.display()))?;

    let mut config = if paths.config_file.exists() {
        load_yaml::<Config>(&paths.config_file)?
    } else {
        let mut generated = Config::default();
        generated.note_roots.push(paths.default_note_root.clone());
        save_config(&paths, &generated)?;
        generated
    };

    validate_schema(&config.schema_version, "config.yml")?;
    if config.note_roots.is_empty() {
        config.note_roots.push(paths.default_note_root.clone());
        save_config(&paths, &config)?;
    }

    for root in &config.note_roots {
        ensure_note_root(root)?;
    }

    let session = if paths.session_file.exists() {
        let loaded = load_yaml::<Session>(&paths.session_file)?;
        validate_schema(&loaded.schema_version, "session.yml")?;
        loaded
    } else {
        let generated = Session::default();
        save_session(&paths, &generated)?;
        generated
    };

    if !paths.activation_file.exists() {
        write_text_atomic(&paths.activation_file, "0\n")?;
    }

    let mut notes = load_notes(&config)?;
    if notes.is_empty() {
        let created = create_note(&config.note_roots[0], &config.history, "", "")?;
        notes.push(created);
    }

    Ok(AppBootstrap {
        messages: load_messages()?,
        paths,
        config,
        session,
        notes,
    })
}

pub fn load_messages() -> Result<MessageCatalog> {
    MessageCatalog::parse(include_str!("../../messages.yml"))
}

pub fn save_config(paths: &AppPaths, config: &Config) -> Result<()> {
    write_yaml(&paths.config_file, config)
}

pub fn save_session(paths: &AppPaths, session: &Session) -> Result<()> {
    write_yaml(&paths.session_file, session)
}

pub fn load_notes(config: &Config) -> Result<Vec<StoredNote>> {
    let mut notes = Vec::new();

    for root in &config.note_roots {
        ensure_note_root(root)?;
        let notes_dir = root.join("notes");
        if !notes_dir.exists() {
            continue;
        }

        for entry in fs::read_dir(&notes_dir)
            .with_context(|| format!("failed to list {}", notes_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension() != Some(OsStr::new("yml")) {
                continue;
            }

            let note: Note = load_yaml(&path)?;
            validate_schema(&note.schema_version, &path.display().to_string())?;
            notes.push(StoredNote {
                root: root.clone(),
                note,
            });
        }
    }

    notes.sort_by(|left, right| {
        right
            .note
            .updated_at
            .cmp(&left.note.updated_at)
            .then_with(|| left.root.cmp(&right.root))
            .then_with(|| left.note.id.cmp(&right.note.id))
    });

    Ok(notes)
}

pub fn create_note(
    root: &Path,
    history: &HistoryConfig,
    title: impl Into<String>,
    content: impl Into<String>,
) -> Result<StoredNote> {
    ensure_note_root(root)?;
    let timestamp = now_rfc3339()?;
    let note = Note {
        schema_version: schema_version_string(),
        id: Ulid::new().to_string(),
        title: title.into(),
        content: content.into(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };

    save_note(root, &note, history)?;

    Ok(StoredNote {
        root: root.to_path_buf(),
        note,
    })
}

pub fn save_note(root: &Path, note: &Note, history: &HistoryConfig) -> Result<()> {
    validate_schema(&note.schema_version, "note")?;
    ensure_note_root(root)?;
    write_yaml(&note_file_path(root, &note.id), note)?;
    append_history_snapshot(root, note)?;
    prune_history(root, &note.id, history.max_snapshots_per_note)?;
    Ok(())
}

pub fn note_file_path(root: &Path, note_id: &str) -> PathBuf {
    root.join("notes").join(format!("{note_id}.yml"))
}

pub fn history_dir(root: &Path, note_id: &str) -> PathBuf {
    root.join("history").join(note_id)
}

pub fn resolve_initial_windows(notes: &[StoredNote], session: &Session) -> Vec<OpenWindowState> {
    let available: BTreeSet<NoteLocator> = notes.iter().map(StoredNote::locator).collect();
    let mut restored = Vec::new();
    let mut seen = BTreeSet::new();

    for window in &session.open_windows {
        let locator = window.locator();
        if available.contains(&locator) && seen.insert(locator.clone()) {
            restored.push(window.clone());
        }
    }

    if restored.is_empty() {
        if let Some(first) = notes.first() {
            restored.push(
                session_open_windows(&[first.locator()], &[default_note_window()])[0].clone(),
            );
        }
    }

    restored
}

pub fn session_open_windows(
    locators: &[NoteLocator],
    placements: &[WindowPlacement],
) -> Vec<OpenWindowState> {
    locators
        .iter()
        .enumerate()
        .map(|(index, locator)| {
            let placement = placements
                .get(index)
                .cloned()
                .unwrap_or_else(default_note_window);
            OpenWindowState::with_placement(locator.clone(), placement)
        })
        .collect()
}

fn validate_schema(schema_version: &str, label: &str) -> Result<()> {
    if schema_version == crate::SCHEMA_VERSION {
        Ok(())
    } else {
        bail!("unsupported schema version {schema_version} in {label}")
    }
}

fn ensure_note_root(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("notes"))
        .with_context(|| format!("failed to create {}", root.display()))?;
    fs::create_dir_all(root.join("history"))
        .with_context(|| format!("failed to create {}", root.display()))?;
    Ok(())
}

fn append_history_snapshot(root: &Path, note: &Note) -> Result<()> {
    let history_path = history_dir(root, &note.id);
    fs::create_dir_all(&history_path)
        .with_context(|| format!("failed to create {}", history_path.display()))?;
    let filename = format!("{}.yml", history_stamp()?);
    write_yaml(&history_path.join(filename), note)
}

fn prune_history(root: &Path, note_id: &str, max_snapshots: usize) -> Result<()> {
    let history_path = history_dir(root, note_id);
    if !history_path.exists() {
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&history_path)
        .with_context(|| format!("failed to list {}", history_path.display()))?
    {
        let entry = entry?;
        if entry.path().extension() == Some(OsStr::new("yml")) {
            entries.push(entry.path());
        }
    }

    entries.sort();
    while entries.len() > max_snapshots {
        let oldest = entries.remove(0);
        fs::remove_file(&oldest)
            .with_context(|| format!("failed to remove {}", oldest.display()))?;
    }

    Ok(())
}

fn write_yaml<T>(path: &Path, value: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let text = serde_yaml::to_string(value)?;
    write_text_atomic(path, &text)
}

fn load_yaml<T>(path: &Path) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(serde_yaml::from_str(&text)?)
}

fn write_text_atomic(path: &Path, text: &str) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let mut temp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp in {parent:?}"))?;
    temp.write_all(text.as_bytes())?;
    temp.flush()?;

    match temp.persist(path) {
        Ok(_) => Ok(()),
        Err(error) => {
            #[cfg(windows)]
            {
                let tempfile = error.file;
                fs::remove_file(path).ok();
                tempfile
                    .persist(path)
                    .map(|_| ())
                    .map_err(|persist_error| persist_error.error.into())
            }
            #[cfg(not(windows))]
            {
                Err(error.error.into())
            }
        }
    }
}

fn now_rfc3339() -> Result<String> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}

fn history_stamp() -> Result<String> {
    Ok(OffsetDateTime::now_utc().format(HISTORY_FILE_STAMP)?)
}

fn app_folder_name() -> &'static str {
    if cfg!(target_os = "linux") {
        "leansticky"
    } else {
        "LeanSticky"
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use tempfile::TempDir;

    use super::{
        AppPaths, Config, HistoryConfig, Note, OpenWindowState, Session, WindowPlacement,
        create_note, history_dir, load_messages, load_notes, load_or_bootstrap_at,
        resolve_initial_windows, save_note,
    };
    use crate::{
        Locale, NoteLocator, WindowPosition, WindowSize, default_management_window,
        default_note_window,
    };

    #[test]
    fn config_round_trip_preserves_fields() {
        let config = Config {
            schema_version: crate::SCHEMA_VERSION.to_owned(),
            locale: Locale::Ja,
            note_roots: vec![PathBuf::from("/tmp/notes")],
            autosave_delay_ms: 2_000,
            history: HistoryConfig {
                max_snapshots_per_note: 42,
            },
        };

        let yaml = serde_yaml::to_string(&config).expect("config should serialize");
        let decoded: Config = serde_yaml::from_str(&yaml).expect("config should deserialize");

        assert_eq!(decoded, config);
    }

    #[test]
    fn session_round_trip_preserves_windows() {
        let session = Session {
            schema_version: crate::SCHEMA_VERSION.to_owned(),
            management_window: default_management_window(),
            open_windows: vec![OpenWindowState {
                root: PathBuf::from("/tmp/notes"),
                note_id: "01TEST".to_owned(),
                position: WindowPosition { x: 1.0, y: 2.0 },
                size: WindowSize {
                    width: 300.0,
                    height: 200.0,
                },
                maximized: true,
            }],
        };

        let yaml = serde_yaml::to_string(&session).expect("session should serialize");
        let decoded: Session = serde_yaml::from_str(&yaml).expect("session should deserialize");

        assert_eq!(decoded, session);
    }

    #[test]
    fn note_round_trip_preserves_fields() {
        let note = Note {
            schema_version: crate::SCHEMA_VERSION.to_owned(),
            id: "01TEST".to_owned(),
            title: "Inbox".to_owned(),
            content: "Body".to_owned(),
            created_at: "2026-03-15T00:00:00Z".to_owned(),
            updated_at: "2026-03-15T00:00:00Z".to_owned(),
        };

        let yaml = serde_yaml::to_string(&note).expect("note should serialize");
        let decoded: Note = serde_yaml::from_str(&yaml).expect("note should deserialize");

        assert_eq!(decoded, note);
    }

    #[test]
    fn bootstrap_creates_files_and_first_note() {
        let fixture = Fixture::new();
        let bootstrap = load_or_bootstrap_at(fixture.paths()).expect("bootstrap should succeed");

        assert!(bootstrap.paths.config_file.exists());
        assert!(bootstrap.paths.session_file.exists());
        assert_eq!(bootstrap.notes.len(), 1);
        assert!(load_messages().is_ok());
    }

    #[test]
    fn history_prunes_to_configured_limit() {
        let fixture = Fixture::new();
        let bootstrap = load_or_bootstrap_at(fixture.paths()).expect("bootstrap should succeed");
        let root = bootstrap.config.note_roots[0].clone();
        let mut note = create_note(
            &root,
            &HistoryConfig {
                max_snapshots_per_note: 2,
            },
            "Title",
            "One",
        )
        .expect("note should be created")
        .note;

        std::thread::sleep(Duration::from_millis(5));
        note.content = "Two".to_owned();
        note.updated_at = "2026-03-15T00:00:01Z".to_owned();
        save_note(
            &root,
            &note,
            &HistoryConfig {
                max_snapshots_per_note: 2,
            },
        )
        .expect("second save should succeed");

        std::thread::sleep(Duration::from_millis(5));
        note.content = "Three".to_owned();
        note.updated_at = "2026-03-15T00:00:02Z".to_owned();
        save_note(
            &root,
            &note,
            &HistoryConfig {
                max_snapshots_per_note: 2,
            },
        )
        .expect("third save should succeed");

        let count = std::fs::read_dir(history_dir(&root, &note.id))
            .expect("history dir should be readable")
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn resolve_windows_filters_missing_notes_and_falls_back() {
        let fixture = Fixture::new();
        let bootstrap = load_or_bootstrap_at(fixture.paths()).expect("bootstrap should succeed");
        let root = bootstrap.config.note_roots[0].clone();
        let created = create_note(&root, &bootstrap.config.history, "Second", "")
            .expect("note should be created");
        let notes = load_notes(&bootstrap.config).expect("notes should load");

        let session = Session {
            management_window: default_management_window(),
            open_windows: vec![
                OpenWindowState::with_placement(
                    created.locator(),
                    WindowPlacement {
                        position: WindowPosition { x: 12.0, y: 20.0 },
                        size: WindowSize {
                            width: 200.0,
                            height: 160.0,
                        },
                        maximized: false,
                    },
                ),
                OpenWindowState::with_placement(
                    NoteLocator::new(root.clone(), "missing"),
                    default_note_window(),
                ),
            ],
            ..Session::default()
        };

        let restored = resolve_initial_windows(&notes, &session);
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].note_id, created.note.id);

        let fallback = resolve_initial_windows(
            &notes,
            &Session {
                open_windows: vec![OpenWindowState::with_placement(
                    NoteLocator::new(root, "missing"),
                    default_note_window(),
                )],
                ..Session::default()
            },
        );
        assert_eq!(fallback.len(), 1);
    }

    struct Fixture {
        _temp: TempDir,
        config_dir: PathBuf,
        data_dir: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = TempDir::new().expect("temp dir should exist");
            let config_dir = temp.path().join("config");
            let data_dir = temp.path().join("data");

            Self {
                _temp: temp,
                config_dir,
                data_dir,
            }
        }

        fn paths(&self) -> AppPaths {
            AppPaths::new(self.config_dir.clone(), self.data_dir.clone())
        }
    }
}
