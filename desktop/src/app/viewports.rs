use eframe::egui;
use egui::{ViewportBuilder, ViewportId};
use leansticky_core::{NoteLocator, WindowPlacement, WindowPosition, WindowSize};

pub(crate) fn native_options(placement: &WindowPlacement, title: &str) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: viewport_builder(title, placement),
        ..Default::default()
    }
}

pub(super) fn viewport_builder(title: &str, placement: &WindowPlacement) -> ViewportBuilder {
    ViewportBuilder::default()
        .with_title(title.to_owned())
        .with_inner_size(egui::vec2(placement.size.width, placement.size.height))
        .with_position(egui::pos2(placement.position.x, placement.position.y))
        .with_maximized(placement.maximized)
}

pub(super) fn viewport_id_for(locator: &NoteLocator) -> ViewportId {
    ViewportId::from_hash_of((locator.root.display().to_string(), locator.note_id.clone()))
}

pub(super) fn capture_current_viewport(
    ctx: &egui::Context,
    placement: &mut WindowPlacement,
) -> bool {
    let (outer_rect, maximized) = ctx.input(|input| {
        let viewport = input.viewport();
        (viewport.outer_rect, viewport.maximized)
    });

    let mut changed = false;

    if let Some(rect) = outer_rect {
        let next_position = WindowPosition {
            x: rect.min.x,
            y: rect.min.y,
        };
        let next_size = WindowSize {
            width: rect.width(),
            height: rect.height(),
        };

        if placement.position != next_position {
            placement.position = next_position;
            changed = true;
        }
        if placement.size != next_size {
            placement.size = next_size;
            changed = true;
        }
    }

    if let Some(maximized) = maximized {
        if placement.maximized != maximized {
            placement.maximized = maximized;
            changed = true;
        }
    }

    changed
}
