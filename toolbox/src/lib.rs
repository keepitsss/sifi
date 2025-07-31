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
    fn try_parse_self(cx: &mut ParsingContext) -> Result<Option<Self>>;

    fn default_case() -> Result<Self>;

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
