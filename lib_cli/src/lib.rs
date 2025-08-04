use std::{collections::BTreeMap, ffi::OsString};

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

#[derive(Debug, Clone, Copy)]
pub struct Documentation {
    pub names: Names,
    pub description: &'static str,
}
#[derive(Debug, Clone, Copy)]
pub struct Names {
    pub main: &'static str,
    pub short: Option<&'static str>,
    pub aliases: &'static [&'static str],
}
#[derive(Debug)]
pub struct DocumentationStore {
    pub item_docs: Documentation,
    pub store: BTreeMap<&'static str, Vec<Documentation>>,
}
mod documentation_impl;

mod router;
pub use router::*;

pub struct EmptyTail;
pub struct TailArgs(pub ParsingContext);
pub mod utils;
use utils::*;

mod parsing;
pub use parsing::*;
