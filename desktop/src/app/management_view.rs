use eframe::egui;
use egui::ViewportCommand;

use super::LeanStickyApp;

impl LeanStickyApp {
    pub(super) fn focus_management_window(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }

    pub(super) fn render_management(&mut self, ctx: &egui::Context) {
        let notes_label = self.text("notes");
        let new_note_label = self.text("new_note");
        let open_windows_label = self.text("open_windows");
        let empty_label = self.text("empty_notes");
        let note_root_label = self.text("note_root");
        let updated_at_label = self.text("updated_at");
        let opened_label = self.text("opened");
        let open_note_label = self.text("open_note");

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(notes_label);
                if ui.button(new_note_label).clicked() {
                    self.new_note();
                }
                ui.label(format!("{open_windows_label}: {}", self.open_order.len()));
            });

            if let Some(message) = &self.banner {
                ui.colored_label(ui.visuals().warn_fg_color, message);
            }

            ui.separator();

            let locators = self.sorted_note_locators();
            if locators.is_empty() {
                ui.label(empty_label);
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for locator in locators {
                    let Some(document) = self.notes.get(&locator) else {
                        continue;
                    };
                    let title = self.note_title(&document.note);
                    let is_open = self.note_is_open(&locator);
                    let updated_at = document.note.updated_at.clone();
                    let root = locator.root.display().to_string();

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(title);
                            if is_open {
                                ui.label(opened_label.clone());
                            } else if ui.button(open_note_label.clone()).clicked() {
                                let placement = self.cascade_placement();
                                self.open_note_with_placement(locator.clone(), placement);
                            }
                        });
                        ui.small(format!("{note_root_label}: {root}"));
                        ui.small(format!("{updated_at_label}: {updated_at}"));
                    });
                }
            });
        });
    }
}
