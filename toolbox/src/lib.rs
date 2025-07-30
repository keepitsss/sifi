use std::{collections::BTreeMap, ffi::OsString};

pub struct ParsingContext {
    pub args: Vec<OsString>,
    pub cursor: usize,
    pub partial_arg_pos: Option<usize>,
    pub documentation: DocumentationStore,
}
#[derive(Default)]
pub struct DocumentationStore {
    pub item_docs: &'static str,
    pub store: BTreeMap<&'static str, Vec<(&'static str, &'static str)>>,
}
impl DocumentationStore {
    pub fn add(&mut self, docs: Documentation) {
        self.store
            .entry(docs.section)
            .or_default()
            .push((docs.association_name, docs.description));
    }
    pub fn build(&self) -> String {
        use std::fmt::Write;

        let mut output = String::new();
        writeln!(&mut output, "{}", self.item_docs).unwrap();
        for (section, items) in &self.store {
            writeln!(&mut output, "\x1b[1;4m{section}s\x1b[0m:").unwrap();

            let least_common_width = items.iter().map(|(name, _desc)| name.len()).max().unwrap();
            for item in items {
                writeln!(
                    &mut output,
                    "      \x1b[1m{name}\x1b[0m{aligning_spaces}  {description}",
                    name = item.0,
                    aligning_spaces = &" ".repeat(least_common_width - item.0.len()),
                    description = item.1
                )
                .unwrap();
            }
        }

        output
    }
}
impl ParsingContext {
    pub fn from_args() -> Self {
        Self {
            args: std::env::args_os().collect(),
            cursor: 1,
            partial_arg_pos: None,
            documentation: DocumentationStore::default(),
        }
    }
}

pub struct Documentation {
    pub section: &'static str,
    pub association_name: &'static str,
    pub description: &'static str,
}
