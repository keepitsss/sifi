use std::collections::HashSet;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(main_page));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

static GLOBAL_STYLES: &str = include_str!("style.css");
use bumpalo::Bump;
use lib_html::*;

async fn main_page() -> axum::response::Html<String> {
    let arena = Bump::new();
    let arena: &Bump = &arena;

    let body = body(arena).child("Hello World!");

    render_page(arena, body).into()
}

fn render_page<'a>(allocator: &'a Bump, body: Body<'a>) -> String {
    let mut html = html(allocator);
    html.body(body);

    let mut cx = Context {
        indentation: utils::Indentation::default(),
        output: String::new(),
        arena: allocator,
        ids: HashSet::new(),
        styles: HashSet::new(),
    };

    cx.styles.extend([GLOBAL_STYLES]);

    html.render(&mut cx);

    let output = cx.output;
    drop(html);
    output
}
