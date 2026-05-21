//! `rusty-grader` — turns captured `cargo` output into a grading [`Verdict`] and maps
//! compiler error codes to the lessons that teach them.
//!
//! Portability contract: this crate is OS-portable. It parses *already-captured* cargo
//! JSON/stdout (`cargo_metadata` is a pure parser, no process/FS) and never spawns
//! cargo itself — `rusty-host` runs the subprocess and passes the output in (§11).

pub mod diagnostic;
pub mod error_map;
pub mod evaluate;
pub mod verdict;

pub use diagnostic::{parse_diagnostics, Diag, Level, Span};
pub use error_map::concept_for_code;
pub use evaluate::{evaluate, CargoOutcome};
pub use verdict::{grade_cargo_test, grade_run_output, verdict_from_diags, Verdict};
