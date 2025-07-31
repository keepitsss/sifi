use std::{collections::BTreeMap, ffi::OsString};

pub struct ParsingContext {
    pub args: Vec<OsString>,
    pub cursor: usize,
    pub documentation: DocumentationStore,
}
impl ParsingContext {
    pub fn from_args() -> Self {
        Self {
            args: std::env::args_os().collect(),
            cursor: 1,
            documentation: DocumentationStore::default(),
        }
    }
    pub fn from_tail(remaining_args: Vec<OsString>) -> Self {
        Self {
            args: remaining_args,
            cursor: 0,
            documentation: DocumentationStore::default(),
        }
    }
}

pub struct Documentation {
    pub section: &'static str,
    pub names: OptNames,
    pub description: &'static str,
}

pub struct OptNames {
    pub main: &'static str,
    pub short: Option<&'static str>,
    pub aliases: &'static [&'static str],
}

#[derive(Default)]
pub struct DocumentationStore {
    pub item_docs: &'static str,
    pub store: BTreeMap<&'static str, Vec<(OptNames, &'static str)>>,
}
impl DocumentationStore {
    pub fn add(&mut self, docs: Documentation) {
        self.store
            .entry(docs.section)
            .or_default()
            .push((docs.names, docs.description));
    }
    pub fn build(&self) -> String {
        use std::fmt::Write;

        let mut output = String::new();
        if !self.item_docs.is_empty() {
            writeln!(&mut output, "{}", self.item_docs).unwrap();
        }
        for (section, items) in &self.store {
            writeln!(&mut output, "\x1b[1;4m{section}s:\x1b[0m").unwrap();

            let least_common_full_name_width = items
                .iter()
                .map(|(names, _desc)| names.main.len())
                .max()
                .unwrap();
            let least_common_short_name_width = items
                .iter()
                .filter_map(|(names, _desc)| names.short)
                .map(|short_name| short_name.len())
                .max();
            // TODO: print aliases
            if let Some(least_common_short_name_width) = least_common_short_name_width {
                for (names, description) in items {
                    writeln!(
                        &mut output,
                        "  \x1b[1m{short_name}{short_aligning_spaces} {name}{main_aligning_spaces}\x1b[0m  {description}",
                        short_name = names.short.map(|x| x.to_owned() + ",").unwrap_or_default(),
                        short_aligning_spaces = &" ".repeat(least_common_short_name_width + 1 - names.short.map(|x| x.len() + 1).unwrap_or_default()),
                        name = names.main,
                        main_aligning_spaces =
                            &" ".repeat(least_common_full_name_width - names.main.len()),
                    )
                    .unwrap();
                }
            } else {
                for (names, description) in items {
                    writeln!(
                        &mut output,
                        "      \x1b[1m{name}{main_aligning_spaces}\x1b[0m  {description}",
                        name = names.main,
                        main_aligning_spaces =
                            &" ".repeat(least_common_full_name_width - names.main.len()),
                    )
                    .unwrap();
                }
            }
        }

        output
    }
}
