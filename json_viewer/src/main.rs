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

    let lines = render_overview(content, structure, 40);
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
                match parent.ty {
                    ObjectType::Array => {
                        // lines.push("],".into());
                    }
                    ObjectType::Structure => {
                        // lines.push("},".into());
                    }
                    _ => unreachable!(),
                }
                current = parent;
            } else {
                break;
            }
        }
    }
    lines
}
