use std::collections::HashSet;

use bumpalo::Bump;
use lib_html::{/* tailwind::TailwindExt,*/ *};

fn main() {
    let allocator = Bump::new();

    let mut cx = Context {
        indentation: utils::Indentation::default(),
        output: String::new(),
        arena: &allocator,
        ids: HashSet::new(),
        styles: HashSet::new(),
    };
    let arena = cx.arena;

    let mut html = html(arena);
    html.body(body(arena).child(example_page(arena)));
    cx.styles.extend([
        "
body {
    background-color: #f0f0f2;
    margin: 0;
    padding: 0;
    font-family: sans-serif;
}
        ",
        "
div {
    width: 600px;
    margin: 5em auto;
    padding: 2em;
    background-color: #fdfdff;
    border-radius: 0.5em;
    box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);
}
        ",
        "
a:link, a:visited {
    color: #38488f;
    text-decoration: none;
}
        ",
        "
@media (max-width: 700px) {
    div {
        margin: 0 auto;
        width: auto;
    }
}
        ",
    ]);

    html.render(&mut cx);

    let output = cx.output;
    drop(html);
    drop(allocator);

    println!("{output}");
    std::fs::write("index.html", output).unwrap();
}

fn example_page(arena: &Bump) -> impl FlowContent<'_> {
    div(arena)
        .child(h1(arena).child("Example Domain"))
        .child(p(arena).child(
            "This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.",
        ))
        .child(p(arena).child(a(arena).href("https://www.iana.org/domains/example").child("More information...")))
}
