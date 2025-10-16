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
1 36 47 9|
 2  9  1 |
7       6|
2 4 3 9 8|
         |
5  9 7  1|
6   5   2|
   7     |
9  8 2  5|
"#;
    let table_element = table(arena).id("sudoku");
    let mut table_element = table_element.body({
        let mut body = tbody(arena);
        for row_ix in 0..3 {
            let mut row = tr(arena);
            for col_ix in 0..9 {
                let cell = grid[1 + row_ix * 11 + col_ix];

                row = row.child(if cell != b' ' {
                    td(arena).child((cell - b'0').to_string())
                } else {
                    td(arena)
                });
            }
            body = body.child(row);
        }
        body
    });
    table_element = table_element.body({
        let mut body = tbody(arena);
        for row_ix in 3..6 {
            let mut row = tr(arena);
            for col_ix in 0..9 {
                let cell = grid[1 + row_ix * 11 + col_ix];

                row = row.child(if cell != b' ' {
                    td(arena).child((cell - b'0').to_string())
                } else {
                    td(arena)
                });
            }
            body = body.child(row);
        }
        body
    });
    table_element = table_element.body({
        let mut body = tbody(arena);
        for row_ix in 6..9 {
            let mut row = tr(arena);
            for col_ix in 0..9 {
                let cell = grid[1 + row_ix * 11 + col_ix];

                row = row.child(if cell != b' ' {
                    td(arena).child((cell - b'0').to_string())
                } else {
                    td(arena)
                });
            }
            body = body.child(row);
        }
        body
    });
    table_element
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
