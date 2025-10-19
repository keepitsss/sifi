use std::collections::HashSet;

use bumpalo::Bump;
use lib_html::{elements::CorrectTableState, *};

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

fn sudoku(arena: &Bump) -> elements::Table<'_, impl CorrectTableState> {
    let grid = br#"
+---------+
|1 36 47 9|
| 2  9  1 |
|7       6|
|2 4 3 9 8|
|         |
|5  9 7  1|
|6   5   2|
|   7     |
|9  8 2  5|
+---------+
"#;
    let cell = |row_ix: usize, col_ix: usize| {
        let temp = grid[14 + row_ix * 12 + col_ix];
        if temp == b' ' {
            td(arena)
        } else {
            td(arena).child((temp - b'0').to_string())
        }
    };

    // FIXME: add `colgroup` and `col` elements
    table(arena).id("sudoku").bodies([
        tbody(arena)
            .rows((0..3).map(|row_ix| tr(arena).cells((0..9).map(|col_ix| cell(row_ix, col_ix))))),
        tbody(arena)
            .rows((3..6).map(|row_ix| tr(arena).cells((0..9).map(|col_ix| cell(row_ix, col_ix))))),
        tbody(arena)
            .rows((6..9).map(|row_ix| tr(arena).cells((0..9).map(|col_ix| cell(row_ix, col_ix))))),
    ])
}

fn example_page(arena: &Bump) -> elements::Body<'_> {
    let header = h(1, arena).child("Example Domain");
    let text = p(arena).child(
            "This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.",
        );
    let link = nav(arena).child(unsafe {
        a(arena)
            .href("https://www.iana.org/domains/example")
            .child("More information...")
    });
    let unordered_list = ul(arena).child(li(arena, NoValue).child("unordered"));
    let ordered_list = ol(arena)
        .start(4)
        .child(li(arena, NoValue).child("1"))
        .child(li(arena, WithValue(3)).child("2"))
        .child(li(arena, NoValue).child("3"))
        .child(li(arena, NoValue).child("4"))
        .marker_type(OrderedListMarkerType::LOWER_ROMAN);
    let lists = figure(arena)
        .child(unordered_list)
        .child(ordered_list)
        .caption(figcaption(arena).child("lists"));

    let sudoku = sudoku(arena);

    body(arena).child(
        // Safety: no other main elements
        unsafe { html_main(arena) }.child(
            div(arena)
                .child(header)
                .child(text)
                .child(link)
                .child(lists)
                .child(sudoku),
        ),
    )
}
