use std::{
    io::{BufWriter, Read, stdin, stdout},
    num::NonZeroU32,
    ops::{Index, IndexMut},
};

macro_rules! measured {
    ($name:literal, $code:expr) => {{
        let start = ::std::time::Instant::now();
        let res = $code;
        let elapsed = start.elapsed();
        eprintln!("{} takes {}ms", $name, elapsed.as_millis());
        res
    }};
}

fn main() -> anyhow::Result<()> {
    let mut content = Vec::new();
    measured!("reading content", stdin().read_to_end(&mut content)?);
    let content: &'static [u8] = content.leak();
    let structure = measured!("parsing structure", {
        use std::hint::black_box;
        for _ in 1..20 {
            parse_json_structure(black_box(content));
        }
        parse_json_structure(content)
    });

    let stdout = BufWriter::new(stdout());

    measured!("rendering", render_data(content, &structure, stdout))?;

    Ok(())
}

const ROOT_INDEX: usize = 0;

fn render_data(
    content: &'static [u8],
    structure: &JsonMetadata,
    mut output: impl std::io::Write,
) -> std::io::Result<()> {
    let mut current_ix = ROOT_INDEX;
    let mut indentation = 0;
    'outer: loop {
        let mut current = structure[current_ix];
        let prefix = if current_ix == ROOT_INDEX {
            String::new()
        } else if let NameOrIndex::Name { start, len } = current.name_or_index {
            format!(
                "{}: ",
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
                writeln!(&mut output, "{prefix}[")?;
                current_ix += 1;
                indentation += 1;
                continue;
            }
            ObjectType::Structure => {
                writeln!(&mut output, "{prefix}{{")?;
                current_ix += 1;
                indentation += 1;
                continue;
            }
            _ => "",
        };
        writeln!(
            &mut output,
            "{prefix}{styles}{}{}",
            str::from_utf8(
                &content[current.source_start..current.source_start + current.source_len]
            )
            .unwrap(),
            if current.next.is_some() && current_ix != ROOT_INDEX {
                ","
            } else {
                ""
            }
        )?;
        loop {
            if current_ix == ROOT_INDEX {
                break 'outer;
            } else if let Some(next) = current.next {
                current_ix = next;
                break;
            } else if let Some(parent_ix) = current.parent {
                let parent = structure[parent_ix];
                indentation -= 1;
                match parent.ty {
                    ObjectType::Array => {
                        let closing = if current_ix == ROOT_INDEX { "]," } else { "]" };
                        writeln!(
                            &mut output,
                            "{indentation}{closing}",
                            indentation = "  ".repeat(indentation)
                        )?;
                    }
                    ObjectType::Structure => {
                        let closing = if current_ix == ROOT_INDEX { "}," } else { "}" };
                        writeln!(
                            &mut output,
                            "{indentation}{closing}",
                            indentation = "  ".repeat(indentation)
                        )?;
                    }
                    _ => unreachable!(),
                }
                if parent_ix == ROOT_INDEX {
                    break 'outer;
                }
                current = parent;
            } else {
                break;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectType {
    String,
    Bool,
    Number,
    EmptyArray,
    EmptyStructure,
    Array,
    Structure,
    Null,
}

#[derive(Debug, Clone, Copy)]
enum NameOrIndex {
    Name { start: usize, len: NonZeroU32 },
    Index,
}
const _: () = assert!(size_of::<NameOrIndex>() == 2 * size_of::<u64>());

#[derive(Debug, Clone, Copy)]
struct ObjectMeta {
    name_or_index: NameOrIndex,
    ty: ObjectType,
    source_start: usize,
    source_len: usize,
    parent: Option<usize>,
    next: Option<usize>,
}

#[derive(Default)]
struct JsonMetadata {
    list: Vec<ObjectMeta>,
}
impl JsonMetadata {
    fn push(&mut self, value: ObjectMeta) -> usize {
        let i = self.list.len();
        self.list.push(value);
        i
    }
}
impl Index<usize> for JsonMetadata {
    type Output = ObjectMeta;
    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}
impl IndexMut<usize> for JsonMetadata {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.list[index]
    }
}

#[derive(Debug, Default)]
enum ParsingState {
    InStructWithName {
        start: usize,
        len: NonZeroU32,
    },
    InStructWithoutName,
    InArray,
    #[default]
    TopLevel,
}

#[derive(Default)]
struct ParsingContext {
    output: JsonMetadata,
    cursor: usize,
    state: ParsingState,
    parent: Option<usize>,
    prev: Option<usize>,
}

fn parse_json_structure(content: &'static [u8]) -> JsonMetadata {
    let mut ctx = ParsingContext::default();
    loop {
        if ctx.cursor == content.len() {
            return ctx.output;
        }
        match content[ctx.cursor] {
            b'[' => {
                let new_index = ctx.add_object_meta(ObjectType::EmptyArray, 0);
                ctx.parent = Some(new_index);
                ctx.prev = None;
                ctx.state = ParsingState::InArray;

                ctx.cursor += 1;
                while content[ctx.cursor].is_ascii_whitespace() {
                    ctx.cursor += 1;
                }
            }
            b'{' => {
                let new_index = ctx.add_object_meta(ObjectType::EmptyStructure, 0);
                ctx.parent = Some(new_index);
                ctx.prev = None;
                ctx.state = ParsingState::InStructWithoutName;

                ctx.cursor += 1;
                while content[ctx.cursor].is_ascii_whitespace() {
                    ctx.cursor += 1;
                }
            }
            b']' => {
                ctx.cursor += 1;

                let parent_ref_mut = &mut ctx.output[ctx.parent.unwrap()];
                parent_ref_mut.source_len = ctx.cursor - parent_ref_mut.source_start;
                match parent_ref_mut.name_or_index {
                    NameOrIndex::Name { .. } => ctx.state = ParsingState::InStructWithoutName,
                    NameOrIndex::Index => ctx.state = ParsingState::InArray,
                }
                ctx.prev = ctx.parent;
                ctx.parent = parent_ref_mut.parent;

                if content.get(ctx.cursor) == Some(&b',') {
                    ctx.cursor += 1;
                }
                while content
                    .get(ctx.cursor)
                    .is_some_and(|c| c.is_ascii_whitespace())
                {
                    ctx.cursor += 1;
                }
            }
            b'}' => {
                ctx.cursor += 1;

                let parent_ref_mut = &mut ctx.output[ctx.parent.unwrap()];
                parent_ref_mut.source_len = ctx.cursor - parent_ref_mut.source_start;
                match parent_ref_mut.name_or_index {
                    NameOrIndex::Name { .. } => ctx.state = ParsingState::InStructWithoutName,
                    NameOrIndex::Index => ctx.state = ParsingState::InArray,
                }
                ctx.prev = ctx.parent;
                ctx.parent = parent_ref_mut.parent;

                if content.get(ctx.cursor) == Some(&b',') {
                    ctx.cursor += 1;
                }
                while content
                    .get(ctx.cursor)
                    .is_some_and(|c| c.is_ascii_whitespace())
                {
                    ctx.cursor += 1;
                }
            }
            b'"' => {
                let len = find_escaped_string_length(unsafe {
                    str::from_utf8_unchecked(&content[ctx.cursor..])
                })
                .unwrap();

                if let ParsingState::InStructWithoutName = ctx.state {
                    ctx.state = ParsingState::InStructWithName {
                        start: ctx.cursor,
                        len: NonZeroU32::new(u32::try_from(len).unwrap()).unwrap(),
                    };
                    ctx.cursor += len;

                    assert_eq!(content[ctx.cursor], b':');
                    ctx.cursor += 1;
                    while content[ctx.cursor].is_ascii_whitespace() {
                        ctx.cursor += 1;
                    }
                } else {
                    let _ = ctx.add_object_meta(ObjectType::String, len);
                    ctx.cursor += len;

                    if content[ctx.cursor] == b',' {
                        ctx.cursor += 1;
                    }
                    while content[ctx.cursor].is_ascii_whitespace() {
                        ctx.cursor += 1;
                    }
                }
            }
            b'-' | b'0'..=b'9' => {
                let mut length = 1;
                while content[ctx.cursor + length].is_ascii_digit() {
                    length += 1;
                }
                if content[ctx.cursor + length] == b'.' {
                    length += 1;
                    while content[ctx.cursor + length].is_ascii_digit() {
                        length += 1;
                    }
                }

                let _ = ctx.add_object_meta(ObjectType::Number, length);

                ctx.cursor += length;
                if content[ctx.cursor] == b',' {
                    ctx.cursor += 1;
                }
                while content[ctx.cursor].is_ascii_whitespace() {
                    ctx.cursor += 1;
                }
            }
            b'n' => {
                assert_eq!(&content[ctx.cursor..ctx.cursor + 4], b"null");

                let _ = ctx.add_object_meta(ObjectType::Null, 4);

                ctx.cursor += 4;
                if content[ctx.cursor] == b',' {
                    ctx.cursor += 1;
                }
                while content[ctx.cursor].is_ascii_whitespace() {
                    ctx.cursor += 1;
                }
            }
            b't' => {
                assert_eq!(&content[ctx.cursor..ctx.cursor + 4], b"true");

                let _ = ctx.add_object_meta(ObjectType::Bool, 4);

                ctx.cursor += 4;
                if content[ctx.cursor] == b',' {
                    ctx.cursor += 1;
                }
                while content[ctx.cursor].is_ascii_whitespace() {
                    ctx.cursor += 1;
                }
            }
            b'f' => {
                assert_eq!(&content[ctx.cursor..ctx.cursor + 5], b"false");

                let _ = ctx.add_object_meta(ObjectType::Bool, 5);

                ctx.cursor += 5;
                if content[ctx.cursor] == b',' {
                    ctx.cursor += 1;
                }
                while content[ctx.cursor].is_ascii_whitespace() {
                    ctx.cursor += 1;
                }
            }
            c => {
                todo!(
                    "unknown character {:?} in symbols {:?}",
                    c as char,
                    str::from_utf8(
                        &content[ctx.output[ctx.parent.unwrap()].source_start..=ctx.cursor + 10]
                    )
                    .unwrap()
                )
            }
        }
    }
}

impl ParsingContext {
    fn add_object_meta(&mut self, ty: ObjectType, source_len: usize) -> usize {
        let meta = self.create_object_meta(ty, source_len);
        self.push_object_meta(meta)
    }

    fn push_object_meta(&mut self, meta: ObjectMeta) -> usize {
        let new_index = self.output.push(meta);
        if let Some(prev) = self.prev {
            assert_ne!(prev, new_index);
            self.output[prev].next = Some(new_index);
        }
        self.prev = Some(new_index);
        new_index
    }

    fn create_object_meta(&mut self, ty: ObjectType, source_len: usize) -> ObjectMeta {
        match &mut self.state {
            ParsingState::InStructWithName { start, len } => {
                let parent = &mut self.output[self.parent.unwrap()].ty;
                assert!(*parent == ObjectType::EmptyStructure || *parent == ObjectType::Structure);
                *parent = ObjectType::Structure;

                let name_or_index = NameOrIndex::Name {
                    start: *start,
                    len: *len,
                };
                let meta = ObjectMeta {
                    name_or_index,
                    ty,
                    source_start: self.cursor,
                    source_len,
                    parent: self.parent,
                    next: None,
                };
                self.state = ParsingState::InStructWithoutName;
                meta
            }
            ParsingState::InStructWithoutName => todo!(),
            ParsingState::InArray => {
                let parent = &mut self.output[self.parent.unwrap()].ty;
                assert!(*parent == ObjectType::EmptyArray || *parent == ObjectType::Array);
                *parent = ObjectType::Array;

                let name_or_index = NameOrIndex::Index;
                ObjectMeta {
                    name_or_index,
                    ty,
                    source_start: self.cursor,
                    source_len,
                    parent: self.parent,
                    next: None,
                }
            }
            ParsingState::TopLevel => {
                // FIXME
                let name_or_index = NameOrIndex::Index;
                ObjectMeta {
                    name_or_index,
                    ty,
                    source_start: self.cursor,
                    source_len,
                    parent: self.parent,
                    next: None,
                }
            }
        }
    }
}

/// Gets escaped string with opening quote and returns bytes count to closing quote(including it)
/// Returns None if closing quote not found
fn find_escaped_string_length(input: &str) -> Option<usize> {
    let mut last_backslash = false;
    for (index, &byte) in input.as_bytes().iter().enumerate().skip(1) {
        match byte {
            b'\\' if !last_backslash => {
                last_backslash = true;
            }
            b'"' if !last_backslash => {
                return Some(index + 1);
            }
            _ => last_backslash = false,
        }
    }
    None
}
