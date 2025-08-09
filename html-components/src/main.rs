use bumpalo::Bump;
use html_components::*;

fn main() {
    let allocator = Bump::new();

    let mut cx = Context {
        indentation: utils::Indentation::default(),
        output: String::new(),
        arena: &allocator,
    };
    let arena = cx.arena;

    let mut html = html(arena);
    let elem = div(arena).id("hi");
    html.add_to_body(div(arena).id("hi").child(elem));

    html.render(&mut cx);

    let output = cx.output;
    drop(html);
    drop(allocator);

    println!("{output}");
}
