use std::{hint::black_box, time::Instant};

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
    let structure = measured!("parsing structure 20 times", {
        for _ in 1..20 {
            parse_json_structure(black_box(content));
        }
        parse_json_structure(content)
    });

    println!("objects: {}", structure.list.len() - 1);
    Ok(())
}
