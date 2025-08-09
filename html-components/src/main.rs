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
    html.add_to_body(div(arena).with_child(div(arena)));

    html.render(&mut cx);

    let output = cx.output;
    drop(html);
    drop(allocator);

    println!("{output}");
}
