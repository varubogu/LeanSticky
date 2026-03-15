use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

pub struct ActivationWatcher {
    path: PathBuf,
    last_modified: Option<SystemTime>,
    last_poll: Instant,
}

impl ActivationWatcher {
    pub fn new(path: PathBuf) -> Result<Self> {
        let last_modified = modified_time(&path);
        Ok(Self {
            path,
            last_modified,
            last_poll: Instant::now(),
        })
    }

    pub fn poll(&mut self) -> bool {
        if self.last_poll.elapsed() < Duration::from_millis(250) {
            return false;
        }
        self.last_poll = Instant::now();

        let Some(modified) = modified_time(&self.path) else {
            return false;
        };

        let changed = self
            .last_modified
            .map(|previous| modified > previous)
            .unwrap_or(true);
        if changed {
            self.last_modified = Some(modified);
        }

        changed
    }
}

pub fn signal_existing_instance(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string();
    fs::write(path, stamp).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn modified_time(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}
