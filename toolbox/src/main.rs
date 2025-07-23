use std::ffi::OsString;

struct _MyArgs {
    link_name: OsString,
    to: OsString,
    destination: OsString,
}

fn main() {
    let var = toolbox::start_parsing().flags3((
        (("hi",), "hello world flag"),
        (("my",), "meeee"),
        (("world", 'w'), "worldldld"),
    ));
    println!("{var:#?}");
    let (_hi, _my, _world) = var.flags;
}
