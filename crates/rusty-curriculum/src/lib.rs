//! `rusty-curriculum` — the typed lesson/exercise model and content parser for Rusty.
//!
//! Portability contract: this crate is OS-portable. No `std::process`, no raw
//! filesystem, no PTY — the filesystem read of `lesson.toml` lives in `rusty-host`,
//! which calls this crate's pure [`parse_lesson`]. Phase 2 implements the model;
//! exercise *rendering/grading* lands in Phase 3.

pub mod loader;
pub mod model;

pub use loader::{parse_lesson, CurriculumError};
pub use model::{
    visible_prefix, Block, CalloutTone, Concept, ConceptId, Exercise, Lesson, LessonId,
    RecallPrompt, Reference, Step, SuccessCriterion, Track,
};
