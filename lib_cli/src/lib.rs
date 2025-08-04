use std::ffi::OsString;

use anyhow::Result;

#[derive(Debug)]
pub struct ParsingContext {
    pub args: Vec<OsString>,
    pub cursor: usize,
    pub documentation: DocumentationStore,
}
impl ParsingContext {
    pub fn from_args(program_docs: Documentation) -> Self {
        Self {
            args: std::env::args_os().collect(),
            cursor: 0,
            documentation: DocumentationStore::new(program_docs),
        }
    }
}
pub trait Opt: Sized {
    /// Returns whether progess is made
    fn try_parse_self(this: &mut Option<Self>, cx: &mut ParsingContext) -> Result<bool>;

    fn finalize(this: Option<Self>) -> Result<Self>;

    const SECTION: &str;
    const DOCUMENTATION: Documentation;
}
pub trait FinalOpt: Sized {
    fn try_parse_self(cx: ParsingContext) -> Result<Self>;
}

mod documentation;
pub use documentation::*;

mod router;
pub use router::*;

pub mod utils;
use utils::*;

mod parsing;
pub use parsing::*;
