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
    fn try_parse_self(cx: &mut ParsingContext) -> Result<Option<Self>>;

    fn default_case() -> Result<Self>;

    const SECTION: &str;
    const DOCUMENTATION: Documentation;
}
pub trait FinalOpt: Sized {
    fn try_parse_self(cx: ParsingContext) -> Result<Self>;
}

#[derive(Debug)]
pub struct DocumentationStore {
    pub item_docs: Documentation,
    pub store: BTreeMap<&'static str, Vec<Documentation>>,
}
#[derive(Debug, Clone, Copy)]
pub struct Documentation {
    pub names: OptNames,
    pub description: &'static str,
}
impl Documentation {
    pub fn todo(main_name: &'static str) -> Self {
        Documentation {
            names: OptNames {
                main: main_name,
                short: None,
                aliases: &[],
            },
            description: "TODO",
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct OptNames {
    pub main: &'static str,
    pub short: Option<&'static str>,
    pub aliases: &'static [&'static str],
}
impl DocumentationStore {
    pub fn new(item_docs: Documentation) -> Self {
        Self {
            item_docs,
            store: BTreeMap::default(),
        }
    }
    pub fn add(&mut self, section: &'static str, docs: Documentation) {
        self.store.entry(section).or_default().push(docs);
    }
    pub fn build(&self) -> String {
        use std::fmt::Write;

        let mut output = String::new();
        {
            // TODO:
            writeln!(
                &mut output,
                "\x1b[1m{}\x1b[0m - {}",
                self.item_docs.names.main, self.item_docs.description
            )
            .unwrap();
        }
        for (section, items) in &self.store {
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "\x1b[1;4m{section}s:\x1b[0m").unwrap();

            let least_common_full_name_width = items
                .iter()
                .map(|docs| docs.names.main.len())
                .max()
                .unwrap();
            let least_common_short_name_width = items
                .iter()
                .filter_map(|docs| docs.names.short)
                .map(|short_name| short_name.len())
                .max();
            for Documentation { names, description } in items {
                let short_name;
                let short_aligning_spaces;
                if let Some(least_common_short_name_width) = least_common_short_name_width {
                    short_name = names.short.map(|x| x.to_owned() + ",").unwrap_or_default();
                    short_aligning_spaces = " ".repeat(
                        least_common_short_name_width + 1
                            - names.short.map(|x| x.len() + 1).unwrap_or_default(),
                    );
                } else {
                    short_name = "".into();
                    short_aligning_spaces = "   ".into();
                }
                let name = names.main;
                let main_aligning_spaces =
                    &" ".repeat(least_common_full_name_width - names.main.len());
                let aliases = if names.aliases.is_empty() {
                    "".into()
                } else {
                    format!("[aliases: {aliases}]", aliases = names.aliases.join(", "))
                };
                writeln!(
                        &mut output,
                        "  \x1b[1m{short_name}{short_aligning_spaces} {name}{main_aligning_spaces}\x1b[0m  {description} {aliases}",
                    )
                    .unwrap();
            }
        }

        output
    }
}
