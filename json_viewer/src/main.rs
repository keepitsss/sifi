use std::time::Instant;

use json_viewer::*;

macro_rules! measured {
    ($name:literal, $code:expr) => {{
        let start = Instant::now();
        let res = $code;
        let elapsed = start.elapsed();
        eprintln!("{} takes {}ms", $name, elapsed.as_millis());
        res
    }};
}

fn main() -> anyhow::Result<()> {
    let content: &'static [u8] = measured!("reading file", {
        std::fs::read_to_string("business-licences.json")?
            .into_bytes()
            .leak()
    });
    let structure = measured!("parsing structure", {
        // use std::hint::black_box;
        // for _ in 1..20 {
        //     parse_json_structure(black_box(content));
        // }
        parse_json_structure(content)
    });

    // let lines = render_overview(content, structure, 40);
    let lines = render_data(content, structure, JsonMetadataIndex::new(1));
    for line in lines {
        println!("{line}");
    }

    // println!("objects: {}", structure.list.len() - 1);
    Ok(())
}

fn render_overview(
    content: &'static [u8],
    structure: JsonMetadata,
    at_least_lines: usize,
) -> Vec<String> {
    const ITALIC: &str = "\x1b[3m";
    const RED_FG: &str = "\x1b[31m";
    const GREEN_FG: &str = "\x1b[32m";
    const BLUE_FG: &str = "\x1b[34m";
    const CYAN_FG: &str = "\x1b[36m";
    const RESET: &str = "\x1b[0m";

    let mut lines = Vec::new();
    let mut current_ix = JsonMetadataIndex::new(0);
    let mut indentation = 0;
    while lines.len() < at_least_lines {
        let mut current = structure[current_ix];
        let prefix = match current.name_or_index {
            NameOrIndex::Name { start, len } => {
                str::from_utf8(&content[(start + 1)..(start + len.get() as usize - 1)])
                    .unwrap()
                    .to_owned()
            }
            NameOrIndex::Index(index) => format!("{index}"),
        };
        let mut prefix = format!(
            "{indentation}{prefix} ",
            indentation = "  ".repeat(indentation)
        );
        match current.ty {
            ObjectType::Array => {
                if indentation > 0 {
                    unsafe {
                        prefix.as_bytes_mut()[indentation * 2 - 2] =
                            if current.expanded { b'-' } else { b'+' };
                    }
                }
                lines.push(format!("{prefix}{CYAN_FG}{ITALIC}arr{RESET}"));
                if current.expanded {
                    current_ix.0 += 1;
                    indentation += 1;
                    continue;
                }
            }
            ObjectType::Structure => {
                if indentation > 0 {
                    unsafe {
                        prefix.as_bytes_mut()[indentation * 2 - 2] =
                            if current.expanded { b'-' } else { b'+' };
                    }
                }
                lines.push(format!("{prefix}{CYAN_FG}{ITALIC}obj{RESET}"));
                if current.expanded {
                    current_ix.0 += 1;
                    indentation += 1;
                    continue;
                }
            }
            ObjectType::EmptyArray => {
                lines.push(format!("{prefix}= []"));
            }
            ObjectType::EmptyStructure => {
                lines.push(format!("{prefix}= {{}}"));
            }
            ObjectType::String => {
                lines.push(format!(
                    "{prefix}{GREEN_FG}{}{RESET}",
                    str::from_utf8(
                        &content[current.source_start..current.source_start + current.source_len]
                    )
                    .unwrap()
                ));
            }
            ObjectType::Null => {
                lines.push(format!("{prefix}{BLUE_FG}null{RESET}",));
            }
            ObjectType::Number => {
                lines.push(format!(
                    "{prefix}{RED_FG}{}{RESET}",
                    str::from_utf8(
                        &content[current.source_start..current.source_start + current.source_len]
                    )
                    .unwrap()
                ));
            }
            _ => {
                lines.push(format!(
                    "{prefix}{}",
                    str::from_utf8(
                        &content[current.source_start..current.source_start + current.source_len]
                    )
                    .unwrap()
                ));
            }
        }
        loop {
            if let Some(next) = current.next {
                current_ix = next;
                break;
            } else if let Some(parent) = current.parent {
                let parent = structure[parent];
                indentation -= 1;
                current = parent;
            } else {
                break;
            }
        }
    }
    lines
}

fn render_data(
    content: &'static [u8],
    structure: JsonMetadata,
    root_ix: JsonMetadataIndex,
) -> Vec<String> {
    const ITALIC: &str = "\x1b[3m";
    const RED_FG: &str = "\x1b[31m";
    const GREEN_FG: &str = "\x1b[32m";
    const YELLOW_FG: &str = "\x1b[33m";
    const BLUE_FG: &str = "\x1b[34m";
    // const CYAN_FG: &str = "\x1b[36m";
    const RESET: &str = "\x1b[0m";

    let mut lines = Vec::new();
    let mut current_ix = root_ix;
    let mut indentation = 0;
    'outer: loop {
        if lines.len() > 40 {
            break;
        }
        let mut current = structure[current_ix];
        let prefix = if let NameOrIndex::Name { start, len } = current.name_or_index {
            format!(
                "{YELLOW_FG}{ITALIC}{}{RESET}: ",
                str::from_utf8(&content[start..start + len.get() as usize])
                    .unwrap()
                    .to_owned()
            )
        } else {
            String::new()
        };
        let prefix = format!(
            "{indentation}{prefix}",
            indentation = "  ".repeat(indentation)
        );
        let styles = match current.ty {
            ObjectType::Array => {
                lines.push(format!("{prefix}["));
                current_ix.0 += 1;
                indentation += 1;
                continue;
            }
            ObjectType::Structure => {
                lines.push(format!("{prefix}{{"));
                current_ix.0 += 1;
                indentation += 1;
                continue;
            }
            ObjectType::EmptyArray => "",
            ObjectType::EmptyStructure => "",
            ObjectType::String => GREEN_FG,
            ObjectType::Bool => {
                todo!();
            }
            ObjectType::Number => RED_FG,
            ObjectType::Null => BLUE_FG,
        };
        lines.push(format!(
            "{prefix}{styles}{}{RESET}{}",
            str::from_utf8(
                &content[current.source_start..current.source_start + current.source_len]
            )
            .unwrap(),
            if current.next.is_some() { "," } else { "" }
        ));
        loop {
            if current_ix == root_ix {
                break 'outer;
            } else if let Some(next) = current.next {
                current_ix = next;
                break;
            } else if let Some(parent_ix) = current.parent {
                let parent = structure[parent_ix];
                indentation -= 1;
                match parent.ty {
                    ObjectType::Array => {
                        lines.push(format!(
                            "{indentation}],",
                            indentation = "  ".repeat(indentation)
                        ));
                    }
                    ObjectType::Structure => {
                        lines.push(format!(
                            "{indentation}}},",
                            indentation = "  ".repeat(indentation)
                        ));
                    }
                    _ => unreachable!(),
                }
                if parent_ix == root_ix {
                    break 'outer;
                }
                current = parent;
            } else {
                break;
            }
        }
    }
    lines
}
