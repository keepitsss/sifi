use std::{
    num::{NonZeroU32, NonZeroU64},
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
}
#[derive(Debug, Clone, Copy)]
struct ObjectMeta {
    name_or_index: NameOrIndex,
    parent: Option<NonZeroU64>,
    ty: ObjectType,
    source_start: u64,
    source_len: u64,
    next: Option<NonZeroU64>,
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

    let mut cursor = 0;
    let mut parent = None;
    let mut name: Option<(u64, u32)> = None;
    let mut prev = None;
    loop {
        if cursor == content.len() {
            eprintln!("TODO: file ending");
            break;
        }
        match content[cursor] {
            b'[' => {
                let new_parent = tree.len() as u64;
                let meta = ObjectMeta {
                    name_or_index: NameOrIndex::Index(0),
                    parent,
                    ty: ObjectType::Array,
                    source_start: cursor as u64,
                    source_len: 0,
                    next: prev,
                };
                tree.push(meta);
                parent = Some(NonZeroU64::new(new_parent).unwrap());
                cursor += 1;
                dbg!(("array started", &tree[1..]));
            }
            b'{' => {
                let new_parent = tree.len() as u64;
                let meta = ObjectMeta {
                    name_or_index: NameOrIndex::Index(0),
                    parent,
                    ty: ObjectType::Structure,
                    source_start: cursor as u64,
                    source_len: 0,
                    next: prev,
                };
                tree.push(meta);
                parent = Some(NonZeroU64::new(new_parent).unwrap());
                cursor += 1;
                dbg!(("structure started", &tree[1..]));
            }
            b'"' => {
                todo!()
            }
            c => todo!("unknown character '{}'", c as char),
        }
    }

    Ok(())
}

/// Gets escaped string without opening quote and returns bytes count to closing quote(including it)
/// Returns None if closing quote not found
fn find_escaped_string_end(input: &str) -> Option<usize> {
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
