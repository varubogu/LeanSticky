mod management_view;
mod note_view;
mod viewports;

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use eframe::egui;
use leansticky_core::{
    AppBootstrap, AppPaths, Config, MessageCatalog, Note, NoteLocator, ResolvedLocale,
    SCHEMA_VERSION, Session, WindowPlacement, create_note, resolve_initial_windows, save_note,
    save_session, session_open_windows,
};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::activation::ActivationWatcher;
use crate::autosave::AutosaveState;

use self::viewports::capture_current_viewport;
pub(crate) use self::viewports::native_options;

pub struct LeanStickyApp {
    paths: AppPaths,
    config: Config,
    messages: MessageCatalog,
    locale: ResolvedLocale,
    management_window: WindowPlacement,
    notes: BTreeMap<NoteLocator, NoteDocument>,
    open_order: Vec<NoteLocator>,
    next_window_index: usize,
    session_dirty: bool,
    last_session_save: Instant,
    activation: ActivationWatcher,
    banner: Option<String>,
}

#[derive(Clone, Debug)]
struct NoteDocument {
    note: Note,
    editor: Option<NoteEditor>,
}

#[derive(Clone, Debug)]
struct NoteEditor {
    placement: WindowPlacement,
    autosave: AutosaveState,
}

impl LeanStickyApp {
    pub fn new(bootstrap: AppBootstrap, activation: ActivationWatcher) -> Self {
        let locale = bootstrap.config.locale.resolve();
        let restored = resolve_initial_windows(&bootstrap.notes, &bootstrap.session);
        let mut notes = BTreeMap::new();
        for stored in bootstrap.notes {
            notes.insert(
                stored.locator(),
                NoteDocument {
                    note: stored.note,
                    editor: None,
                },
            );
        }

        let mut app = Self {
            banner: (!bootstrap.session.open_windows.is_empty()
                && restored.len() != bootstrap.session.open_windows.len())
            .then(|| bootstrap.messages.text(locale, "restore_empty")),
            paths: bootstrap.paths,
            config: bootstrap.config,
            messages: bootstrap.messages,
            locale,
            management_window: bootstrap.session.management_window,
            notes,
            open_order: Vec::new(),
            next_window_index: 0,
            session_dirty: false,
            last_session_save: Instant::now(),
            activation,
        };

        for restored_window in restored {
            app.open_note_with_placement(restored_window.locator(), restored_window.placement());
        }

        if app.open_order.is_empty() {
            if let Some(first) = app.sorted_note_locators().into_iter().next() {
                let placement = app.cascade_placement();
                app.open_note_with_placement(first, placement);
            }
        }

        app.session_dirty = true;
        app
    }

    fn text(&self, key: &str) -> String {
        self.messages.text(self.locale, key)
    }

    fn note_title(&self, note: &Note) -> String {
        let trimmed = note.title.trim();
        if trimmed.is_empty() {
            self.text("untitled_note")
        } else {
            trimmed.to_owned()
        }
    }

    fn sorted_note_locators(&self) -> Vec<NoteLocator> {
        let mut entries: Vec<_> = self
            .notes
            .iter()
            .map(|(locator, document)| {
                (
                    locator.clone(),
                    document.note.updated_at.clone(),
                    document.note.title.clone(),
                )
            })
            .collect();
        entries.sort_by(|left, right| {
            right
                .1
                .cmp(&left.1)
                .then_with(|| left.2.cmp(&right.2))
                .then_with(|| left.0.cmp(&right.0))
        });
        entries.into_iter().map(|entry| entry.0).collect()
    }

    fn note_is_open(&self, locator: &NoteLocator) -> bool {
        self.notes
            .get(locator)
            .and_then(|document| document.editor.as_ref())
            .is_some()
    }

    fn cascade_placement(&mut self) -> WindowPlacement {
        let mut placement = leansticky_core::default_note_window();
        let offset = (self.next_window_index % 6) as f32 * 24.0;
        placement.position.x += offset;
        placement.position.y += offset;
        self.next_window_index += 1;
        placement
    }

    fn open_note_with_placement(&mut self, locator: NoteLocator, placement: WindowPlacement) {
        let Some(document) = self.notes.get_mut(&locator) else {
            return;
        };
        if document.editor.is_none() {
            document.editor = Some(NoteEditor {
                placement,
                autosave: AutosaveState::default(),
            });
            self.open_order.retain(|open| open != &locator);
            self.open_order.push(locator);
            self.session_dirty = true;
        }
    }

    fn new_note(&mut self) {
        let Some(root) = self.config.note_roots.first().cloned() else {
            self.banner = Some("No note root configured.".to_owned());
            return;
        };

        match create_note(&root, &self.config.history, "", "") {
            Ok(created) => {
                let locator = created.locator();
                self.notes.insert(
                    locator.clone(),
                    NoteDocument {
                        note: created.note,
                        editor: None,
                    },
                );
                let placement = self.cascade_placement();
                self.open_note_with_placement(locator, placement);
            }
            Err(error) => {
                self.banner = Some(format!("{} {error}", self.text("save_failed")));
            }
        }
    }

    fn save_note_now(&mut self, locator: &NoteLocator) {
        let Some(document) = self.notes.get(locator) else {
            return;
        };

        let mut note = document.note.clone();
        note.updated_at = current_timestamp();
        let result = save_note(&locator.root, &note, &self.config.history);
        let save_failed_prefix = self.text("save_failed");

        if let Some(document) = self.notes.get_mut(locator) {
            if let Some(editor) = document.editor.as_mut() {
                match result {
                    Ok(()) => {
                        document.note = note;
                        editor.autosave.mark_saved();
                        self.session_dirty = true;
                    }
                    Err(error) => {
                        let message = format!("{save_failed_prefix} {error}");
                        editor.autosave.mark_failed(message.clone());
                        self.banner = Some(message);
                    }
                }
            }
        }
    }

    fn save_note_if_dirty(&mut self, locator: &NoteLocator) {
        let dirty = self
            .notes
            .get(locator)
            .and_then(|document| document.editor.as_ref())
            .is_some_and(|editor| editor.autosave.dirty());
        if dirty {
            self.save_note_now(locator);
        }
    }

    fn close_note(&mut self, locator: &NoteLocator) {
        self.save_note_if_dirty(locator);
        if let Some(document) = self.notes.get_mut(locator) {
            document.editor = None;
        }
        self.open_order.retain(|open| open != locator);
        self.session_dirty = true;
    }

    fn flush_all(&mut self) {
        let locators = self.open_order.clone();
        for locator in locators {
            self.save_note_if_dirty(&locator);
        }
        self.save_session_now();
    }

    fn save_session_now(&mut self) {
        let session = self.build_session();
        match save_session(&self.paths, &session) {
            Ok(()) => {
                self.session_dirty = false;
                self.last_session_save = Instant::now();
            }
            Err(error) => {
                self.banner = Some(format!("{} {error}", self.text("session_save_failed")));
            }
        }
    }

    fn maybe_save_session(&mut self) {
        if self.session_dirty && self.last_session_save.elapsed() >= Duration::from_millis(250) {
            self.save_session_now();
        }
    }

    fn build_session(&self) -> Session {
        let placements: Vec<_> = self
            .open_order
            .iter()
            .filter_map(|locator| {
                self.notes
                    .get(locator)
                    .and_then(|document| document.editor.as_ref())
                    .map(|editor| editor.placement.clone())
            })
            .collect();
        let open_windows = session_open_windows(&self.open_order, &placements);

        Session {
            schema_version: SCHEMA_VERSION.to_owned(),
            management_window: self.management_window.clone(),
            open_windows,
        }
    }

    fn next_repaint_after(&self) -> Duration {
        let delay = Duration::from_millis(self.config.autosave_delay_ms);
        let now = Instant::now();
        let mut next = Duration::from_millis(250);

        for document in self.notes.values() {
            let Some(editor) = document.editor.as_ref() else {
                continue;
            };
            let Some(pending_since) = editor.autosave.pending_since() else {
                continue;
            };

            if editor.autosave.dirty() {
                let elapsed = now.saturating_duration_since(pending_since);
                if elapsed >= delay {
                    return Duration::from_millis(16);
                }

                let remaining = delay - elapsed;
                if remaining < next {
                    next = remaining;
                }
            }
        }

        next
    }
}

impl eframe::App for LeanStickyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.activation.poll() {
            self.focus_management_window(ctx);
        }

        if capture_current_viewport(ctx, &mut self.management_window) {
            self.session_dirty = true;
        }

        if ctx.input(|input| input.viewport().close_requested()) {
            self.flush_all();
        }

        self.render_management(ctx);
        self.render_note_viewports(ctx);
        self.maybe_save_session();
        ctx.request_repaint_after(self.next_repaint_after());
    }
}

fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}
