use std::{collections::BTreeMap, ffi::OsString};

pub struct ParsingContext {
    pub args: Vec<OsString>,
    pub cursor: usize,
    pub partial_arg_pos: Option<usize>,
}
impl ParsingContext {
    pub fn from_args() -> Self {
        Self {
            args: std::env::args_os().collect(),
            cursor: 1,
            partial_arg_pos: None,
        }
    }
}

pub struct Documentation {
    pub section: &'static str,
    pub description: &'static str,
}

pub fn add_documentation(
    docs: Documentation,
    store: &mut BTreeMap<&'static str, Vec<&'static str>>,
) {
    store
        .entry(docs.section)
        .or_default()
        .push(docs.description);
}
pub fn build_documentation(
    main_description: &'static str,
    opts: BTreeMap<&'static str, Vec<&'static str>>,
) -> String {
    let mut output = String::new();
    output.push_str(main_description);
    output.push('\n');
    for (section, items) in opts {
        output.push_str(section);
        output.push_str(":\n");
        for item in items {
            output.push_str("    ");
            output.push_str(item);
            output.push('\n');
        }
    }

    output
}
