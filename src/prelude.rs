//! Convenient re-exports for the most commonly used items.
//!
//! ```
//! use rsx_a11y::prelude::*;
//! ```

pub use crate::dom::{Aria, AriaValueType, AttributeName, Role, Tag};
pub use crate::lints::{LintDiagnostic, Rule, Severity};
pub use crate::parser::{ParseError, ParseErrorKind};
pub use crate::{
    LintOptions, LintSummary, check_files, check_project, check_project_with, check_source,
};
