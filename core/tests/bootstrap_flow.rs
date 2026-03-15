use std::path::PathBuf;

use leansticky_core::{
    AppPaths, OpenWindowState, Session, WindowPlacement, WindowPosition, WindowSize, create_note,
    load_notes, load_or_bootstrap_at, resolve_initial_windows,
};
use tempfile::TempDir;

#[test]
fn bootstrap_and_restore_flow_works_with_temp_dirs() {
    let fixture = Fixture::new();
    let bootstrap = load_or_bootstrap_at(fixture.paths()).expect("bootstrap should succeed");
    let root = bootstrap.config.note_roots[0].clone();
    let created = create_note(&root, &bootstrap.config.history, "Scratch", "body")
        .expect("new note should be created");
    let notes = load_notes(&bootstrap.config).expect("notes should load");

    let restored = resolve_initial_windows(
        &notes,
        &Session {
            open_windows: vec![OpenWindowState::with_placement(
                created.locator(),
                WindowPlacement {
                    position: WindowPosition { x: 10.0, y: 20.0 },
                    size: WindowSize {
                        width: 320.0,
                        height: 200.0,
                    },
                    maximized: false,
                },
            )],
            ..Session::default()
        },
    );

    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].note_id, created.note.id);
    assert!(bootstrap.paths.config_file.exists());
    assert!(bootstrap.paths.session_file.exists());
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
