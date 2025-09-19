use std::{
    hint::black_box,
    num::NonZeroU32,
    ops::{Index, IndexMut},
    time::Instant,
};

macro_rules! measured {
    ($name:literal, $code:expr) => {{
        let start = Instant::now();
        let res = $code;
        let elapsed = start.elapsed();
        eprintln!("{} takes {}ms", $name, elapsed.as_millis());
        res
    }};
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
struct JsonMetadataIndex(u32);
impl JsonMetadataIndex {
    fn new(index: usize) -> Self {
        JsonMetadataIndex(u32::try_from(index).unwrap())
    }
    fn get(self) -> usize {
        self.0 as usize
    }
}

#[derive(Default)]
struct JsonMetadata {
    list: Vec<ObjectMeta>,
}
impl JsonMetadata {
    fn push(&mut self, value: ObjectMeta) -> JsonMetadataIndex {
        let i = self.list.len();
        self.list.push(value);
        JsonMetadataIndex::new(i)
    }
}
impl Index<JsonMetadataIndex> for JsonMetadata {
    type Output = ObjectMeta;
    fn index(&self, index: JsonMetadataIndex) -> &Self::Output {
        &self.list[index.get()]
    }
}
impl IndexMut<JsonMetadataIndex> for JsonMetadata {
    fn index_mut(&mut self, index: JsonMetadataIndex) -> &mut Self::Output {
        &mut self.list[index.get()]
    }
}

#[derive(Debug, Clone, Copy)]
struct ObjectMeta {
    name_or_index: NameOrIndex,
    ty: ObjectType,
    source_start: usize,
    source_len: usize,
    parent: Option<JsonMetadataIndex>,
    prev: Option<JsonMetadataIndex>,
    next: Option<JsonMetadataIndex>,
}

#[derive(Debug, Clone, Copy)]
enum NameOrIndex {
    Name { start: u64, len: NonZeroU32 },
    Index(u64),
}
const _: () = assert!(size_of::<NameOrIndex>() == 2 * size_of::<u64>());

#[derive(Debug, Default)]
enum ParsingState {
    InStructWithName {
        start: u64,
        len: NonZeroU32,
    },
    InStructWithoutName,
    InArray {
        index: u64,
    },
    #[default]
    TopLevel,
}

fn main() -> anyhow::Result<()> {
    let content: &'static [u8] = measured!("reading file", {
        std::fs::read_to_string("business-licences.json")?
            .into_bytes()
            .leak()
    });
    let structure = measured!("parsing structure 20 times", {
        for _ in 1..20 {
            parse_json_structure(black_box(content));
        }
        parse_json_structure(content)
    });
    println!("objects: {}", structure.list.len() - 1);
    Ok(())
}

#[derive(Default)]
struct ParsingContext {
    output: JsonMetadata,
    cursor: usize,
    state: ParsingState,
    parent: Option<JsonMetadataIndex>,
    prev: Option<JsonMetadataIndex>,
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
                ctx.state = ParsingState::InArray { index: 0 };

                ctx.cursor += 1;
            }
            b'{' => {
                let new_index = ctx.add_object_meta(ObjectType::EmptyStructure, 0);
                ctx.parent = Some(new_index);
                ctx.prev = None;
                ctx.state = ParsingState::InStructWithoutName;

                ctx.cursor += 1;
            }
            b']' => {
                ctx.cursor += 1;

                let parent_ref_mut = &mut ctx.output[ctx.parent.unwrap()];
                parent_ref_mut.source_len = ctx.cursor - parent_ref_mut.source_start;
                match parent_ref_mut.name_or_index {
                    NameOrIndex::Name { .. } => ctx.state = ParsingState::InStructWithoutName,
                    NameOrIndex::Index(index) => {
                        ctx.state = ParsingState::InArray { index: index + 1 }
                    }
                }
                ctx.prev = ctx.parent;
                ctx.parent = parent_ref_mut.parent;

                if content.get(ctx.cursor) == Some(&b',') {
                    ctx.cursor += 1;
                }
                while content.get(ctx.cursor) == Some(&b' ') {
                    ctx.cursor += 1;
                }
            }
            b'}' => {
                ctx.cursor += 1;

                let parent_ref_mut = &mut ctx.output[ctx.parent.unwrap()];
                parent_ref_mut.source_len = ctx.cursor - parent_ref_mut.source_start;
                match parent_ref_mut.name_or_index {
                    NameOrIndex::Name { .. } => ctx.state = ParsingState::InStructWithoutName,
                    NameOrIndex::Index(index) => {
                        ctx.state = ParsingState::InArray { index: index + 1 }
                    }
                }
                ctx.prev = ctx.parent;
                ctx.parent = parent_ref_mut.parent;

                if content.get(ctx.cursor) == Some(&b',') {
                    ctx.cursor += 1;
                }
                while content.get(ctx.cursor) == Some(&b' ') {
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
                        start: ctx.cursor as u64,
                        len: NonZeroU32::new(u32::try_from(len).unwrap()).unwrap(),
                    };
                    ctx.cursor += len;

                    assert_eq!(content[ctx.cursor], b':');
                    ctx.cursor += 1;
                    while content[ctx.cursor] == b' ' {
                        ctx.cursor += 1;
                    }
                } else {
                    let _ = ctx.add_object_meta(ObjectType::String, len);
                    ctx.cursor += len;

                    if content[ctx.cursor] == b',' {
                        ctx.cursor += 1;
                    }
                    while content[ctx.cursor] == b' ' {
                        ctx.cursor += 1;
                    }
                }
            }
            b'n' => {
                assert_eq!(&content[ctx.cursor..ctx.cursor + 4], b"null");

                let _ = ctx.add_object_meta(ObjectType::Null, 4);

                ctx.cursor += 4;
                if content[ctx.cursor] == b',' {
                    ctx.cursor += 1;
                }
                while content[ctx.cursor] == b' ' {
                    ctx.cursor += 1;
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
                while content[ctx.cursor] == b' ' {
                    ctx.cursor += 1;
                }
            }
            c => {
                todo!(
                    "unknown character '{}' in symbols '{}'",
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
    fn add_object_meta(&mut self, ty: ObjectType, source_len: usize) -> JsonMetadataIndex {
        let meta = self.create_object_meta(ty, source_len);
        self.push_object_meta(meta)
    }

    fn push_object_meta(&mut self, meta: ObjectMeta) -> JsonMetadataIndex {
        let new_index = self.output.push(meta);
        if let Some(prev) = self.prev {
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
                    prev: self.prev,
                    next: None,
                };
                self.state = ParsingState::InStructWithoutName;
                meta
            }
            ParsingState::InStructWithoutName => todo!(),
            ParsingState::InArray { index } => {
                let parent = &mut self.output[self.parent.unwrap()].ty;
                assert!(*parent == ObjectType::EmptyArray || *parent == ObjectType::Array);
                *parent = ObjectType::Array;

                let name_or_index = NameOrIndex::Index(*index);
                *index += 1;
                ObjectMeta {
                    name_or_index,
                    ty,
                    source_start: self.cursor,
                    source_len,
                    parent: self.parent,
                    prev: self.prev,
                    next: None,
                }
            }
            ParsingState::TopLevel => {
                // FIXME
                let name_or_index = NameOrIndex::Index(0);
                ObjectMeta {
                    name_or_index,
                    ty,
                    source_start: self.cursor,
                    source_len,
                    parent: self.parent,
                    prev: self.prev,
                    next: None,
                }
            }
        }
    }
}

/// Gets escaped string withopening quote and returns bytes count to closing quote(including it)
/// Returns None if closing quote not found
fn find_escaped_string_length(input: &str) -> Option<usize> {
    memchr::memchr_iter(b'"', input.as_bytes().get(0..)?)
        .skip(1) // opening quote
        .find(|quote_ix| input.as_bytes()[quote_ix - 1] != b'\\')
        .map(|quote_ix| quote_ix + 1)
}
