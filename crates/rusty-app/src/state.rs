use rusty_curriculum::{ConceptId, LessonId};
use rusty_scheduler::ReviewState;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The persistent application state saved to `.rusty-state/progress.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistentState {
    /// Lessons that the learner has fully completed.
    pub completed_lessons: HashSet<LessonId>,
    /// SM-2 review state for individual concepts.
    pub concept_reviews: HashMap<ConceptId, ReviewState>,
}

impl PersistentState {
    /// The default location for the progress file: `<repo-root>/.rusty-state/progress.json`.
    pub fn default_path() -> PathBuf {
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push(".rusty-state");
        path.push("progress.json");
        path
    }

    /// Load the state from the given path. If the file doesn't exist, returns default state.
    pub fn load(path: &Path) -> Self {
        if let Ok(contents) = fs::read_to_string(path) {
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save the state to the given path, creating parent directories if needed.
    pub fn save(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    /// Update the state of a concept after a review.
    pub fn update_review(&mut self, concept: ConceptId, quality: u8, now: u64) {
        let state = self.concept_reviews.entry(concept).or_default();
        *state = rusty_scheduler::grade_review(state, quality, now);
    }
}
