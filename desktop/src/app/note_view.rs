use std::time::{Duration, Instant};

use eframe::egui;
use egui::ViewportClass;
use leansticky_core::NoteLocator;

use super::LeanStickyApp;
use super::viewports::{capture_current_viewport, viewport_builder, viewport_id_for};

impl LeanStickyApp {
    pub(super) fn render_note_viewports(&mut self, ctx: &egui::Context) {
        let open_notes = self.open_order.clone();
        for locator in open_notes {
            let Some(document) = self.notes.get(&locator) else {
                continue;
            };
            let Some(editor) = document.editor.as_ref() else {
                continue;
            };
            let title = self.note_title(&document.note);
            let builder = viewport_builder(&title, &editor.placement);
            let viewport_id = viewport_id_for(&locator);

            ctx.show_viewport_immediate(viewport_id, builder, |viewport_ctx, class| {
                self.render_note_contents(viewport_ctx, class, locator.clone());
            });
        }
    }

    pub(super) fn render_note_contents(
        &mut self,
        viewport_ctx: &egui::Context,
        class: ViewportClass,
        locator: NoteLocator,
    ) {
        if !self.note_is_open(&locator) {
            return;
        }

        if capture_current_viewport(viewport_ctx, self.editor_placement_mut(&locator)) {
            self.session_dirty = true;
        }

        let mut close_requested = viewport_ctx.input(|input| input.viewport().close_requested());
        let close_label = self.text("close_note");
        let title_hint = self.text("title_placeholder");
        let saved_status = self.text("status_saved");
        let pending_status = self.text("status_pending");
        let error_status = self.text("status_error");
        let autosave_waiting = self.text("autosave_waiting");
        let mut should_save = false;

        let mut render_ui = |ui: &mut egui::Ui, this: &mut Self| {
            let now = Instant::now();
            let delay = Duration::from_millis(this.config.autosave_delay_ms);
            let Some(document) = this.notes.get_mut(&locator) else {
                return;
            };
            let Some(editor) = document.editor.as_mut() else {
                return;
            };

            let status_text = if editor.autosave.last_error().is_some() {
                error_status.clone()
            } else if editor.autosave.dirty() {
                pending_status.clone()
            } else {
                saved_status.clone()
            };

            ui.horizontal(|ui| {
                if ui.button(close_label.clone()).clicked() {
                    close_requested = true;
                }
                ui.label(status_text);
            });
            ui.small(autosave_waiting.clone());
            ui.separator();

            let title_changed = ui
                .add(
                    egui::TextEdit::singleline(&mut document.note.title)
                        .hint_text(title_hint.clone()),
                )
                .changed();
            ui.separator();
            let content_size = ui.available_size();
            let content_changed = ui
                .add_sized(
                    content_size,
                    egui::TextEdit::multiline(&mut document.note.content)
                        .desired_width(f32::INFINITY),
                )
                .changed();

            if title_changed || content_changed {
                editor.autosave.mark_changed(now);
                this.session_dirty = true;
            }

            if editor.autosave.due(now, delay) {
                should_save = true;
            }
        };

        match class {
            ViewportClass::Embedded => {
                let mut open = true;
                egui::Window::new(
                    self.note_title(&self.notes.get(&locator).expect("note should exist").note),
                )
                .open(&mut open)
                .show(viewport_ctx, |ui| render_ui(ui, self));
                if !open {
                    close_requested = true;
                }
            }
            _ => {
                egui::CentralPanel::default().show(viewport_ctx, |ui| render_ui(ui, self));
            }
        }

        if should_save {
            self.save_note_now(&locator);
        }

        if close_requested {
            self.close_note(&locator);
        }
    }

    fn editor_placement_mut(
        &mut self,
        locator: &NoteLocator,
    ) -> &mut leansticky_core::WindowPlacement {
        &mut self
            .notes
            .get_mut(locator)
            .expect("note should exist")
            .editor
            .as_mut()
            .expect("note should be open")
            .placement
    }
}
