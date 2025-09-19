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
    let mut lines = Vec::new();
    let mut current_ix = JsonMetadataIndex::new(0);
    let mut indentation = 0;
    while lines.len() < 30 {
        let mut current = structure[current_ix];
        let prefix = if let NameOrIndex::Name { start, len } = current.name_or_index {
            str::from_utf8(&content[start..start + len.get() as usize])
                .unwrap()
                .to_owned()
                + ": "
        } else {
            String::new()
        };
        let prefix = format!(
            "{indentation}{prefix}",
            indentation = "  ".repeat(indentation)
        );
        match current.ty {
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
            _ => (),
        }
        lines.push(format!(
            "{prefix}{},",
            str::from_utf8(
                &content[current.source_start..current.source_start + current.source_len]
            )
            .unwrap()
        ));
        loop {
            if let Some(next) = current.next {
                current_ix = next;
                break;
            } else if let Some(parent) = current.parent {
                let parent = structure[parent];
                indentation -= 1;
                match parent.ty {
                    ObjectType::Array => {
                        lines.push("],".into());
                    }
                    ObjectType::Structure => {
                        lines.push("},".into());
                    }
                    _ => unreachable!(),
                }
                current = parent;
            } else {
                break;
            }
        }
    }
    for line in lines {
        println!("{line}");
    }

    // println!("objects: {}", structure.list.len() - 1);
    Ok(())
}
