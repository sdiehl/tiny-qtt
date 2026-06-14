//! Tiny quantitative type theory checker.

// These categories conflict with the "no doc novels, math-style names"
// house rules; everything else clippy flags is treated as an error.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::doc_markdown
)]

pub mod diagnostics;
pub mod driver;
pub mod elab;
pub mod errors;
pub mod eval;
pub mod lexer;
pub mod mult;
pub mod parse;
pub mod pretty;
pub mod syntax;
pub mod value;

pub use driver::check_str;
pub use errors::{TinyQttError, TinyQttResult};
