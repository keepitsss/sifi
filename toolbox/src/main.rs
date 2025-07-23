use std::ffi::OsString;

use toolbox::flags::*;

struct _MyArgs {
    link_name: OsString,
    to: OsString,
    destination: OsString,
}

fn main() {
    let var = toolbox::start_parsing().flags(((("hi",), "hello world flag"),));
    println!("{var:#?}");
    todo!();
}
