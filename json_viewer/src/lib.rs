use std::{
    num::NonZeroU32,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
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
pub enum NameOrIndex {
    Name { start: usize, len: NonZeroU32 },
    Index(u64),
}
const _: () = assert!(size_of::<NameOrIndex>() == 2 * size_of::<u64>());

#[derive(Debug, Clone, Copy)]
pub struct ObjectMeta {
    pub name_or_index: NameOrIndex,
    pub ty: ObjectType,
    pub source_start: usize,
    pub source_len: usize,
    pub parent: Option<JsonMetadataIndex>,
    pub prev: Option<JsonMetadataIndex>,
    pub next: Option<JsonMetadataIndex>,

    pub expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonMetadataIndex(pub u32);
impl JsonMetadataIndex {
    pub fn new(index: usize) -> Self {
        JsonMetadataIndex(u32::try_from(index).unwrap())
    }
    pub fn get(self) -> usize {
        self.0 as usize
    }
}

#[derive(Default)]
pub struct JsonMetadata {
    pub list: Vec<ObjectMeta>,
}
impl JsonMetadata {
    fn push(&mut self, value: ObjectMeta) -> JsonMetadataIndex {
        let i = self.list.len();
        self.list.push(value);
        JsonMetadataIndex::new(i)
    }
    pub fn next_visible(&self, index: JsonMetadataIndex) -> Option<JsonMetadataIndex> {
        let object = self[index];
        match object.ty {
            ObjectType::Array | ObjectType::Structure if object.expanded => {
                Some(JsonMetadataIndex::new(index.get() + 1))
            }
            _ => self.next_no_children(index),
        }
    }
    fn next_no_children(&self, index: JsonMetadataIndex) -> Option<JsonMetadataIndex> {
        let object = self[index];
        if let Some(next) = object.next {
            Some(next)
        } else {
            self.next_no_children(object.parent?)
        }
    }
    pub fn prev_visible(&self, index: JsonMetadataIndex) -> Option<JsonMetadataIndex> {
        let object = self[index];
        if let Some(prev) = object.prev {
            Some(self.last_in_childrens(prev))
        } else {
            object.parent
        }
    }
    fn last_in_childrens(&self, index: JsonMetadataIndex) -> JsonMetadataIndex {
        let object = self[index];
        match object.ty {
            ObjectType::Array | ObjectType::Structure if object.expanded => {
                let mut last = JsonMetadataIndex::new(index.get() + 1);

                while let Some(next) = self[last].next {
                    last = next;
                }
                self.last_in_childrens(last)
            }
            _ => index,
        }
    }
    pub fn depth(&self, index: JsonMetadataIndex) -> usize {
        if let Some(parent) = self[index].parent {
            self.depth(parent) + 1
        } else {
            0
        }
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
    InArray {
        index: u64,
    },
    #[default]
    TopLevel,
}

#[derive(Default)]
pub struct ParsingContext {
    output: JsonMetadata,
    cursor: usize,
    state: ParsingState,
    parent: Option<JsonMetadataIndex>,
    prev: Option<JsonMetadataIndex>,
}

pub fn parse_json_structure(content: &'static [u8]) -> JsonMetadata {
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
                    NameOrIndex::Index(index) => {
                        ctx.state = ParsingState::InArray { index: index + 1 }
                    }
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
                    NameOrIndex::Index(index) => {
                        ctx.state = ParsingState::InArray { index: index + 1 }
                    }
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
    fn add_object_meta(&mut self, ty: ObjectType, source_len: usize) -> JsonMetadataIndex {
        let meta = self.create_object_meta(ty, source_len);
        self.push_object_meta(meta)
    }

    fn push_object_meta(&mut self, meta: ObjectMeta) -> JsonMetadataIndex {
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
                    prev: self.prev,
                    next: None,
                    expanded: false,
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
                    expanded: false,
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
                    expanded: true,
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
