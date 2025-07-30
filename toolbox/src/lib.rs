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
    pub fn build(self) -> String {
        let mut output = String::new();
        output.push_str(self.item_docs);
        output.push('\n');
        for (section, items) in self.store {
            output.push_str(section);
            output.push_str(":\n");

            let least_common_width = items.iter().map(|(name, _desc)| name.len()).max().unwrap();
            for item in items {
                output.push_str("      ");
                output.push_str(item.0);
                output.push_str(&" ".repeat(least_common_width - item.0.len() + 2));
                output.push_str(item.1);
                output.push('\n');
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
