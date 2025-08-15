use std::collections::{HashMap, HashSet};

use bumpalo::Bump;
use lib_html::*;

fn main() {
    let allocator = Bump::new();

    let mut cx = Context {
        indentation: utils::Indentation::default(),
        output: String::new(),
        arena: &allocator,
        ids: HashSet::new(),
        tailwind_styles: HashMap::new(),
    };
    let arena = cx.arena;

    let mut html = html(arena);
    html.add_to_body(
        div(arena)
            .id("id2")
            .class("some-class")
            .class("other")
            .font_sans()
            .classes(["some1", "some2"])
            .child(div(arena).id("id1").child("hi")),
    );
    //     html.head.add_style(
    //         "
    // body {
    //     background-color: #f0f0f2;
    //     margin: 0;
    //     padding: 0;
    //     font-family: sans-serif;
    // }
    //         ",
    //     );
    //     html.head.add_style(
    //         "
    // div {
    //     width: 600px;
    //     margin: 5em auto;
    //     padding: 2em;
    //     background-color: #fdfdff;
    //     border-radius: 0.5em;
    //     box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);
    // }
    //         ",
    //     );
    //     html.head.add_style(
    //         "
    // a:link, a:visited {
    //     color: #38488f;
    //     text-decoration: none;
    // }
    //         ",
    //     );
    //     html.head.add_style(
    //         "
    // @media (max-width: 700px) {
    //     div {
    //         margin: 0 auto;
    //         width: auto;
    //     }
    // }
    //         ",
    //     );

    html.render(&mut cx);

    let output = cx.output;
    // drop(html);
    drop(allocator);

    println!("{output}");
    std::fs::write("index.html", output).unwrap();
}
