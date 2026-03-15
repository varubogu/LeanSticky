use std::time::{Duration, Instant};

#[derive(Clone, Debug, Default)]
pub struct AutosaveState {
    dirty: bool,
    pending_since: Option<Instant>,
    last_error: Option<String>,
}

impl AutosaveState {
    pub fn mark_changed(&mut self, now: Instant) {
        self.dirty = true;
        self.pending_since = Some(now);
        self.last_error = None;
    }

    pub fn due(&self, now: Instant, delay: Duration) -> bool {
        self.dirty
            && self
                .pending_since
                .is_some_and(|started| now.saturating_duration_since(started) >= delay)
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn pending_since(&self) -> Option<Instant> {
        self.pending_since
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn mark_saved(&mut self) {
        self.dirty = false;
        self.pending_since = None;
        self.last_error = None;
    }

    pub fn mark_failed(&mut self, message: impl Into<String>) {
        self.last_error = Some(message.into());
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::AutosaveState;

    #[test]
    fn autosave_transitions_from_clean_to_dirty_to_saved() {
        let now = Instant::now();
        let later = now + Duration::from_millis(1_600);
        let mut state = AutosaveState::default();

        assert!(!state.dirty());
        state.mark_changed(now);
        assert!(state.dirty());
        assert!(!state.due(
            now + Duration::from_millis(200),
            Duration::from_millis(1_500)
        ));
        assert!(state.due(later, Duration::from_millis(1_500)));

        state.mark_saved();
        assert!(!state.dirty());
        assert!(state.pending_since().is_none());
    }

    #[test]
    fn autosave_retains_dirty_state_on_failure() {
        let now = Instant::now();
        let mut state = AutosaveState::default();

        state.mark_changed(now);
        state.mark_failed("save failed");

        assert!(state.dirty());
        assert_eq!(state.last_error(), Some("save failed"));
    }
}
