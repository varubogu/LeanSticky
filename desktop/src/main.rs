mod activation;
mod app;
mod autosave;
mod fonts;

use leansticky_core::load_or_bootstrap;
use single_instance::SingleInstance;

use crate::activation::{ActivationWatcher, signal_existing_instance};
use crate::app::{LeanStickyApp, native_options};
use crate::fonts::install_cjk_fallback_font;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let bootstrap = load_or_bootstrap()?;
    let instance = SingleInstance::new("LeanStickyDesktop")?;
    if !instance.is_single() {
        signal_existing_instance(&bootstrap.paths.activation_file)?;
        return Ok(());
    }

    let title = format!(
        "{} {}",
        bootstrap
            .messages
            .text(bootstrap.config.locale.resolve(), "app_title"),
        bootstrap
            .messages
            .text(bootstrap.config.locale.resolve(), "management_window")
    );
    let options = native_options(&bootstrap.session.management_window, &title);
    let activation = ActivationWatcher::new(bootstrap.paths.activation_file.clone())?;

    eframe::run_native(
        &title,
        options,
        Box::new(move |creation_context| {
            install_cjk_fallback_font(&creation_context.egui_ctx);
            Ok(Box::new(LeanStickyApp::new(bootstrap, activation)))
        }),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    Ok(())
}
