use std::collections::HashSet;

use bumpalo::Bump;
use html_components::*;

fn main() {
    let allocator = Bump::new();

    let mut cx = Context {
        indentation: utils::Indentation::default(),
        output: String::new(),
        arena: &allocator,
        ids: HashSet::new(),
    };
    let arena = cx.arena;

    let mut html = html(arena);
    let elem = div(arena).id("id1");
    html.add_to_body(div(arena).id("id2").child(elem));

    html.render(&mut cx);

    let output = cx.output;
    drop(html);
    drop(allocator);

    println!("{output}");
}
