mod messages;
mod model;
mod storage;

pub use messages::{LocalizedMessage, MessageCatalog};
pub use model::{
    Config, HistoryConfig, Locale, Note, NoteLocator, OpenWindowState, ResolvedLocale,
    SCHEMA_VERSION, Session, StoredNote, WindowPlacement, WindowPosition, WindowSize,
    default_autosave_delay_ms, default_management_window, default_note_window,
};
pub use storage::{
    AppBootstrap, AppPaths, create_note, default_note_root, history_dir, load_messages, load_notes,
    load_or_bootstrap, load_or_bootstrap_at, note_file_path, resolve_initial_windows, save_config,
    save_note, save_session, session_open_windows,
};
