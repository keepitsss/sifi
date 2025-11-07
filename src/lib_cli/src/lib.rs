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
    /// Returns whether progress is made
    fn try_parse_self(this: &mut Option<Self>, cx: &mut ParsingContext) -> Result<bool>;

    fn finalize(this: Option<Self>) -> Result<Self>;

    fn add_documentation(store: &mut DocumentationStore);
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
impl Names {
    pub const fn only_main(name: &'static str) -> Self {
        Names {
            main: name,
            short: None,
            aliases: &[],
        }
    }
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

macro_rules! all_the_tuples_named {
    ($inner_macro:ident) => {
        $inner_macro!([(T1, n1)], R);
        $inner_macro!([(T1, n1), (T2, n2)], R);
        $inner_macro!([(T1, n1), (T2, n2), (T3, n3)], R);
        $inner_macro!([(T1, n1), (T2, n2), (T3, n3), (T4, n4)], R);
        $inner_macro!([(T1, n1), (T2, n2), (T3, n3), (T4, n4), (T5, n5)], R);
        $inner_macro!(
            [(T1, n1), (T2, n2), (T3, n3), (T4, n4), (T5, n5), (T6, n6)],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7)
            ],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7),
                (T8, n8)
            ],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7),
                (T8, n8),
                (T9, n9)
            ],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7),
                (T8, n8),
                (T9, n9),
                (T10, n10)
            ],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7),
                (T8, n8),
                (T9, n9),
                (T10, n10),
                (T11, n11)
            ],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7),
                (T8, n8),
                (T9, n9),
                (T10, n10),
                (T11, n11),
                (T12, n12)
            ],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7),
                (T8, n8),
                (T9, n9),
                (T10, n10),
                (T11, n11),
                (T12, n12),
                (T13, n13)
            ],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7),
                (T8, n8),
                (T9, n9),
                (T10, n10),
                (T11, n11),
                (T12, n12),
                (T13, n13),
                (T14, n14)
            ],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7),
                (T8, n8),
                (T9, n9),
                (T10, n10),
                (T11, n11),
                (T12, n12),
                (T13, n13),
                (T14, n14),
                (T15, n15)
            ],
            R
        );
        $inner_macro!(
            [
                (T1, n1),
                (T2, n2),
                (T3, n3),
                (T4, n4),
                (T5, n5),
                (T6, n6),
                (T7, n7),
                (T8, n8),
                (T9, n9),
                (T10, n10),
                (T11, n11),
                (T12, n12),
                (T13, n13),
                (T14, n14),
                (T15, n15),
                (T16, n16)
            ],
            R
        );
    };
}

mod parsing;
pub use parsing::*;

mod sequence;
pub use sequence::*;
