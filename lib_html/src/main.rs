use std::collections::HashSet;

use bumpalo::Bump;
use lib_html::*;

static GLOBAL_STYLES: &str = include_str!("style.css");

fn main() {
    let allocator = Bump::new();

    let mut html = html(&allocator);
    html.body(example_page(&allocator));

    let mut cx = Context {
        indentation: utils::Indentation::default(),
        output: String::new(),
        arena: &allocator,
        ids: HashSet::new(),
        styles: HashSet::new(),
    };

    cx.styles.extend([GLOBAL_STYLES]);

    html.render(&mut cx);

    let output = cx.output;
    drop(html);
    drop(allocator);

    println!("{output}");
    std::fs::write("index.html", output).unwrap();
}

fn example_page(arena: &Bump) -> Body<'_> {
    let header = h1(arena).child("Example Domain");
    let text = p(arena).child(
            "This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.",
        );
    let link = p(arena).child(
        a(arena)
            .href("https://www.iana.org/domains/example")
            .child("More information..."),
    );
    body(arena).child(div(arena).child(header).child(text).child(link))
}
