use std::{
    num::{NonZeroU32, NonZeroUsize},
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
#[repr(u64)]
enum ObjectType {
    String = 1,
    Bool = 2,
    Number = 3,
    EmptyArray = 4,
    EmptyStructure = 5,
    Array = 6,
    Structure = 7,
    Null = 8,
}
#[derive(Debug, Clone, Copy)]
struct ObjectMeta {
    name_or_index: NameOrIndex,
    parent: Option<NonZeroUsize>,
    ty: ObjectType,
    source_start: usize,
    source_len: usize,
    next: Option<NonZeroUsize>,
}

#[derive(Debug, Clone, Copy)]
enum NameOrIndex {
    Name { start: u64, len: NonZeroU32 },
    Index(u64),
}
const _: () = assert!(size_of::<NameOrIndex>() == 2 * size_of::<u64>());

#[derive(Debug)]
enum Context {
    InStructWithName { start: u64, len: NonZeroU32 },
    InStructWithoutName,
    InArray { index: u64 },
    TopLevel,
}

fn main() -> anyhow::Result<()> {
    let content: &'static [u8] = measured!("reading file", {
        std::fs::read_to_string("business-licences.json")?
            .into_bytes()
            .leak()
    });
    let mut tree: Vec<ObjectMeta> = Vec::new();
    tree.push({
        // FIXME: use zeroed value
        ObjectMeta {
            name_or_index: NameOrIndex::Index(0),
            parent: None,
            ty: ObjectType::Bool,
            source_start: 0,
            source_len: 0,
            next: None,
        }
    }); // So indexes start from 1

    let application_start = Instant::now();
    let mut cursor = 0;
    let mut parent = None;
    let mut context = Context::TopLevel;
    let mut prev = None;
    loop {
        if cursor == content.len() {
            eprintln!("TODO: file ending");
            break;
        }
        match content[cursor] {
            b'[' => {
                let meta_prototype = |name_or_index| ObjectMeta {
                    name_or_index,
                    parent,
                    ty: ObjectType::EmptyArray,
                    source_start: cursor,
                    source_len: 0,
                    next: prev,
                };
                let meta = finish_object_meta(&mut tree, parent, &mut context, meta_prototype);
                let new_index = push_object_meta(&mut tree, &mut prev, meta);
                parent = Some(new_index);
                context = Context::InArray { index: 0 };

                cursor += 1;
            }
            b'{' => {
                let meta_prototype = |name_or_index| ObjectMeta {
                    name_or_index,
                    parent,
                    ty: ObjectType::EmptyStructure,
                    source_start: cursor,
                    source_len: 0,
                    next: None,
                };
                let meta = finish_object_meta(&mut tree, parent, &mut context, meta_prototype);
                let new_index = push_object_meta(&mut tree, &mut prev, meta);
                parent = Some(new_index);
                context = Context::InStructWithoutName;

                cursor += 1;
            }
            b'"' => {
                let start = cursor;
                cursor += 1;
                let len = find_escaped_string_length(str::from_utf8(&content[cursor..]).unwrap())
                    .unwrap()
                    + 2;
                cursor += len - 1;

                if let Context::InStructWithoutName = context {
                    context = Context::InStructWithName {
                        start: start as u64,
                        len: NonZeroU32::new(u32::try_from(len).unwrap()).unwrap(),
                    };

                    assert_eq!(content[cursor], b':');
                    cursor += 1;
                    while content[cursor] == b' ' {
                        cursor += 1;
                    }
                } else {
                    let meta_prototype = |name_or_index| ObjectMeta {
                        name_or_index,
                        parent,
                        ty: ObjectType::String,
                        source_start: start,
                        source_len: len,
                        next: None,
                    };
                    let meta = finish_object_meta(&mut tree, parent, &mut context, meta_prototype);
                    let _ = push_object_meta(&mut tree, &mut prev, meta);

                    if content[cursor] == b',' {
                        cursor += 1;
                    }
                    while content[cursor] == b' ' {
                        cursor += 1;
                    }
                }
            }
            b'n' => {
                assert_eq!(&content[cursor..cursor + 4], b"null");

                let meta_prototype = |name_or_index| ObjectMeta {
                    name_or_index,
                    parent,
                    ty: ObjectType::Null,
                    source_start: cursor,
                    source_len: 4,
                    next: None,
                };
                let meta = finish_object_meta(&mut tree, parent, &mut context, meta_prototype);
                let _ = push_object_meta(&mut tree, &mut prev, meta);
                cursor += 4;

                if content[cursor] == b',' {
                    cursor += 1;
                }
                while content[cursor] == b' ' {
                    cursor += 1;
                }
            }
            b'-' | b'0'..=b'9' => {
                let start = cursor;
                cursor += 1;
                while content[cursor].is_ascii_digit() {
                    cursor += 1;
                }
                if content[cursor] == b'.' {
                    cursor += 1;
                    while content[cursor].is_ascii_digit() {
                        cursor += 1;
                    }
                }

                let meta_prototype = |name_or_index| ObjectMeta {
                    name_or_index,
                    parent,
                    ty: ObjectType::Number,
                    source_start: start,
                    source_len: cursor - start,
                    next: None,
                };
                let meta = finish_object_meta(&mut tree, parent, &mut context, meta_prototype);
                let _ = push_object_meta(&mut tree, &mut prev, meta);

                if content[cursor] == b',' {
                    cursor += 1;
                }
                while content[cursor] == b' ' {
                    cursor += 1;
                }
            }
            c => {
                todo!(
                    "unknown character '{}' in symbols '{}'",
                    c as char,
                    str::from_utf8(
                        &content[tree[parent.unwrap().get()].source_start..=cursor + 10]
                    )
                    .unwrap()
                )
            }
        }
        if cursor > 10000 {
            println!(
                "cursor: {}, elapsed: {}, micros per byte: {}",
                cursor,
                application_start.elapsed().as_millis(),
                application_start.elapsed().as_secs_f64() * 1_000_000. / cursor as f64,
            );
            dbg!(&tree[1..10]);
            break;
        }
    }

    Ok(())
}

fn push_object_meta(
    tree: &mut Vec<ObjectMeta>,
    prev: &mut Option<std::num::NonZero<usize>>,
    meta: ObjectMeta,
) -> std::num::NonZero<usize> {
    let new_index = NonZeroUsize::new(tree.len()).unwrap();
    tree.push(meta);
    if let Some(prev) = *prev {
        tree[prev.get()].next = Some(new_index);
    }
    *prev = Some(new_index);
    new_index
}

fn finish_object_meta(
    tree: &mut [ObjectMeta],
    parent: Option<NonZeroUsize>,
    context: &mut Context,
    meta_prototype: impl Fn(NameOrIndex) -> ObjectMeta,
) -> ObjectMeta {
    match context {
        Context::InStructWithName { start, len } => {
            let parent = &mut tree[parent.unwrap().get()].ty;
            assert!(*parent == ObjectType::EmptyStructure || *parent == ObjectType::Structure);
            *parent = ObjectType::Structure;

            let name_or_index = NameOrIndex::Name {
                start: *start,
                len: *len,
            };
            let meta = meta_prototype(name_or_index);
            *context = Context::InStructWithoutName;
            meta
        }
        Context::InStructWithoutName => todo!(),
        Context::InArray { index } => {
            let parent = &mut tree[parent.unwrap().get()].ty;
            assert!(*parent == ObjectType::EmptyArray || *parent == ObjectType::Array);
            *parent = ObjectType::Array;

            let name_or_index = NameOrIndex::Index(*index);
            *index += 1;
            meta_prototype(name_or_index)
        }
        Context::TopLevel => {
            // FIXME
            let name_or_index = NameOrIndex::Index(0);
            meta_prototype(name_or_index)
        }
    }
}

/// Gets escaped string without opening quote and returns bytes count to closing quote(including it)
/// Returns None if closing quote not found
fn find_escaped_string_length(input: &str) -> Option<usize> {
    let mut last_backslash = false;
    for (index, &byte) in input.as_bytes().iter().enumerate() {
        match byte {
            b'/' if !last_backslash => {
                last_backslash = true;
            }
            b'"' if !last_backslash => {
                return Some(index);
            }
            _ => last_backslash = false,
        }
    }
    None
}
