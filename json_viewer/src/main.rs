use std::{
    num::{NonZeroU32, NonZeroU64, NonZeroUsize},
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

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
enum ObjectType {
    String = 1,
    Bool = 2,
    Number = 3,
    Structure = 4,
    Array = 5,
    Null = 6,
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

fn main() -> anyhow::Result<()> {
    let content: &'static [u8] = measured!("reading file", {
        std::fs::read_to_string("business-licences.json")?
            .into_bytes()
            .leak()
    });
    let mut tree: Vec<ObjectMeta> = Vec::new();
    tree.push(unsafe {
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
    let mut name: Option<(u64, NonZeroU32)> = None;
    let mut prev = None;
    loop {
        if cursor == content.len() {
            eprintln!("TODO: file ending");
            break;
        }
        match content[cursor] {
            b'[' => {
                let new_parent = tree.len();
                let meta = ObjectMeta {
                    name_or_index: NameOrIndex::Index(0),
                    parent,
                    ty: ObjectType::Array,
                    source_start: cursor,
                    source_len: 0,
                    next: prev,
                };
                tree.push(meta);
                parent = Some(NonZeroUsize::new(new_parent).unwrap());
                cursor += 1;
            }
            b'{' => {
                let new_parent = tree.len();

                let meta = if let Some((name_start, name_len)) = name {
                    ObjectMeta {
                        name_or_index: NameOrIndex::Name {
                            start: name_start,
                            len: name_len,
                        },
                        parent,
                        ty: ObjectType::Structure,
                        source_start: cursor,
                        source_len: 0,
                        next: None,
                    }
                } else if let ObjectMeta {
                    ty: ObjectType::Array,
                    name_or_index,
                    ..
                } = &mut tree[parent.unwrap().get()]
                {
                    let NameOrIndex::Index(index) = name_or_index else {
                        unreachable!()
                    };
                    let my_index = *index;
                    *index += 1;
                    ObjectMeta {
                        name_or_index: NameOrIndex::Index(my_index),
                        parent,
                        ty: ObjectType::Structure,
                        source_start: cursor,
                        source_len: 0,
                        next: None,
                    }
                } else {
                    dbg!(tree[parent.unwrap().get()]);
                    unreachable!()
                };
                let new_index = NonZeroUsize::new(tree.len()).unwrap();
                tree.push(meta);
                if let Some(prev) = prev {
                    tree[prev.get()].next = Some(new_index);
                }
                prev = Some(new_index);
                name = None;
                parent = Some(NonZeroUsize::new(new_parent).unwrap());

                cursor += 1;
            }
            b'"' => {
                let start = cursor;
                cursor += 1;
                let len = find_escaped_string_length(str::from_utf8(&content[cursor..]).unwrap())
                    .unwrap()
                    + 2;
                cursor += len - 1;
                // dbg!(str::from_utf8(&content[start..start + len]).unwrap());
                // dbg!(tree[parent.unwrap().get()]);
                // dbg!(name);
                if let Some((name_start, name_len)) = name {
                    let meta = ObjectMeta {
                        name_or_index: NameOrIndex::Name {
                            start: name_start,
                            len: name_len,
                        },
                        parent,
                        ty: ObjectType::String,
                        source_start: start,
                        source_len: len,
                        next: None,
                    };
                    let new_index = NonZeroUsize::new(tree.len()).unwrap();
                    tree.push(meta);
                    if let Some(prev) = prev {
                        tree[prev.get()].next = Some(new_index);
                    }
                    prev = Some(new_index);
                    name = None;
                    if content[cursor] == b',' {
                        cursor += 1;
                    }
                    while content[cursor] == b' ' {
                        cursor += 1;
                    }
                } else {
                    match &mut tree[parent.unwrap().get()] {
                        ObjectMeta {
                            ty: ObjectType::Structure,
                            ..
                        } => {
                            name = Some((
                                start as u64,
                                NonZeroU32::new(u32::try_from(len).unwrap()).unwrap(),
                            ));
                            assert_eq!(content[cursor], b':');
                            cursor += 1;
                            while content[cursor] == b' ' {
                                cursor += 1;
                            }
                        }
                        ObjectMeta {
                            ty: ObjectType::Array,
                            name_or_index,
                            ..
                        } => {
                            let NameOrIndex::Index(index) = name_or_index else {
                                unreachable!();
                            };
                            let my_index = *index;
                            *index += 1;

                            let meta = ObjectMeta {
                                name_or_index: NameOrIndex::Index(my_index),
                                parent,
                                ty: ObjectType::String,
                                source_start: start,
                                source_len: len,
                                next: None,
                            };

                            let new_index = NonZeroUsize::new(tree.len()).unwrap();
                            tree.push(meta);

                            if let Some(prev) = prev {
                                tree[prev.get()].next = Some(new_index);
                            }
                            prev = Some(new_index);

                            if content[cursor] == b',' {
                                cursor += 1;
                            }
                            while content[cursor] == b' ' {
                                cursor += 1;
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            b'n' => {
                assert_eq!(&content[cursor..cursor + 4], b"null");
                let meta = if let Some((name_start, name_len)) = name {
                    ObjectMeta {
                        name_or_index: NameOrIndex::Name {
                            start: name_start,
                            len: name_len,
                        },
                        parent,
                        ty: ObjectType::Null,
                        source_start: cursor,
                        source_len: 4,
                        next: None,
                    }
                } else if let ObjectMeta {
                    ty: ObjectType::Array,
                    name_or_index,
                    ..
                } = &mut tree[parent.unwrap().get()]
                {
                    let NameOrIndex::Index(index) = name_or_index else {
                        unreachable!()
                    };
                    let my_index = *index;
                    *index += 1;
                    ObjectMeta {
                        name_or_index: NameOrIndex::Index(my_index),
                        parent,
                        ty: ObjectType::Null,
                        source_start: cursor,
                        source_len: 4,
                        next: None,
                    }
                } else {
                    unreachable!()
                };
                let new_index = NonZeroUsize::new(tree.len()).unwrap();
                tree.push(meta);
                if let Some(prev) = prev {
                    tree[prev.get()].next = Some(new_index);
                }
                prev = Some(new_index);
                name = None;
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
                let meta = if let Some((name_start, name_len)) = name {
                    ObjectMeta {
                        name_or_index: NameOrIndex::Name {
                            start: name_start,
                            len: name_len,
                        },
                        parent,
                        ty: ObjectType::Number,
                        source_start: start,
                        source_len: cursor - start,
                        next: None,
                    }
                } else if let ObjectMeta {
                    ty: ObjectType::Array,
                    name_or_index,
                    ..
                } = &mut tree[parent.unwrap().get()]
                {
                    let NameOrIndex::Index(index) = name_or_index else {
                        unreachable!()
                    };
                    let my_index = *index;
                    *index += 1;
                    ObjectMeta {
                        name_or_index: NameOrIndex::Index(my_index),
                        parent,
                        ty: ObjectType::Number,
                        source_start: start,
                        source_len: cursor - start,
                        next: None,
                    }
                } else {
                    unreachable!()
                };
                let new_index = NonZeroUsize::new(tree.len()).unwrap();
                tree.push(meta);
                if let Some(prev) = prev {
                    tree[prev.get()].next = Some(new_index);
                }
                prev = Some(new_index);
                name = None;

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
