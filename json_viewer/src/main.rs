use std::{
    fs::File,
    io::{Read, Write},
    time::Instant,
};

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
    let _alternate_screen_wrapper =
        alternate_screen_wrapper::unix::AlternateScreenOnStdout::enter()?.unwrap();

    let mut stdout = std::io::stdout();

    // hide cursor
    stdout.write_all(b"\x1B[?25l")?;
    // disable line wrap
    stdout.write_all(b"\x1B[?7l")?;

    let mut scroll = 0;
    let mut selection = 0;

    let mut stdin = std::io::stdin();
    let mut buf = [0; 1024];
    loop {
        let height = render_frame(content, &structure, &mut stdout, scroll, selection)?;
        stdout.flush()?;

        let count = stdin.read(&mut buf)?;
        if count == 0 {
            unreachable!("it's okay to delete this line, just checking");
            // std::thread::sleep_ms(100);
            // continue;
        }
        if buf[..count].contains(&b'q') {
            break;
        }
        let down = buf[..count].iter().filter(|&&key| key == b'j').count();
        let up = buf[..count].iter().filter(|&&key| key == b'k').count();
        selection = selection.saturating_add(down);
        selection = selection.saturating_sub(up);
        if selection < scroll {
            scroll = selection;
        }
        if selection >= scroll + height as usize {
            scroll = selection - height as usize + 1;
        }
    }

    Ok(())
}

/// # Returns terminal height
fn render_frame(
    content: &'static [u8],
    structure: &JsonMetadata,
    stdout: &mut std::io::Stdout,
    scroll: usize,
    selection: usize,
) -> anyhow::Result<u16> {
    // clear screen
    stdout.write_all(b"\x1B[2J")?;

    let rustix::termios::Winsize {
        ws_row: height,
        ws_col: width,
        ..
    } = rustix::termios::tcgetwinsize(File::open("/dev/tty")?)?;

    let lines = render_overview(content, structure, height as usize + scroll, selection);
    for (i, line) in lines
        .into_iter()
        .skip(scroll)
        .enumerate()
        .take_while(|(i, _)| *i <= height as usize)
    {
        let height_gap = 0;
        let width_gap = 0;
        // move cursor
        stdout.write_fmt(format_args!(
            "\x1B[{};{}H",
            i as u16 + height_gap + 1, // height
            width_gap + 1,             // width
        ))?;
        stdout.write_all(line.as_bytes())?;
        let max_width = (width * 2 / 5) as usize;
        stdout.write_all(&b" ".repeat(max_width))?;
        // move cursor
        stdout.write_fmt(format_args!(
            "\x1B[{};{}H",
            i as u16 + height_gap + 1, // height
            max_width + width_gap + 1, // width
        ))?;
        stdout.write_all(RESET.as_bytes())?;
        // move cursor
        stdout.write_fmt(format_args!(
            "\x1B[{};{}H",
            i as u16 + height_gap + 1, // height
            max_width + width_gap + 1, // width
        ))?;
        stdout.write_all(b"\x1b[K|")?;
    }
    Ok(height)
}

const ITALIC: &str = "\x1b[3m";
const RED_FG: &str = "\x1b[31m";
const GREEN_FG: &str = "\x1b[32m";
const BLUE_FG: &str = "\x1b[34m";
const CYAN_FG: &str = "\x1b[36m";
const BLUE_BG: &str = "\x1b[44m";
// const MAGENTA_BG: &str = "\x1b[45m";
const RESET: &str = "\x1b[0m";
fn render_overview(
    content: &'static [u8],
    structure: &JsonMetadata,
    at_least_lines: usize,
    selection: usize,
) -> Vec<String> {
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
        let current_line_selected = lines.len() == selection;
        let red_fg = if current_line_selected { "" } else { RED_FG };
        let green_fg = if current_line_selected { "" } else { GREEN_FG };
        let blue_fg = if current_line_selected { "" } else { BLUE_FG };
        let cyan_fg = if current_line_selected { "" } else { CYAN_FG };
        let mut prefix = format!(
            "{selection}{indentation}{prefix} ",
            selection = if current_line_selected { BLUE_BG } else { "" },
            indentation = "  ".repeat(indentation)
        );
        match current.ty {
            ObjectType::Array => {
                if indentation > 0 {
                    unsafe {
                        prefix.as_bytes_mut()
                            [indentation * 2 - 2 + 5 * current_line_selected as usize] =
                            if current.expanded { b'-' } else { b'+' };
                    }
                }
                lines.push(format!("{prefix}{cyan_fg}{ITALIC}arr"));
                if current.expanded {
                    current_ix.0 += 1;
                    indentation += 1;
                    continue;
                }
            }
            ObjectType::Structure => {
                if indentation > 0 {
                    unsafe {
                        prefix.as_bytes_mut()
                            [indentation * 2 - 2 + 5 * current_line_selected as usize] =
                            if current.expanded { b'-' } else { b'+' };
                    }
                }
                lines.push(format!("{prefix}{cyan_fg}{ITALIC}obj"));
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
                    "{prefix}{green_fg}{}",
                    str::from_utf8(
                        &content[current.source_start..current.source_start + current.source_len]
                    )
                    .unwrap()
                ));
            }
            ObjectType::Null => {
                lines.push(format!("{prefix}{blue_fg}null"));
            }
            ObjectType::Number => {
                lines.push(format!(
                    "{prefix}{red_fg}{}",
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
    structure: &JsonMetadata,
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
