use anyhow::Result;
use lib_cli::*;

fn main() -> Result<()> {
    let cx = ParsingContext::from_args(Documentation {
        names: Names {
            main: "test_program",
            short: None,
            aliases: &[],
        },
        description: "command line parsing library",
    });
    cx.wrapper(|utils::AppPath(_path), TailArgs(args)| {
        args.subcommand(
            Documentation::todo("subcmd"),
            |FlagHi(is_hi_set), FlagWorld(is_world_set), EmptyTail| {
                println!("is_hi_set: {is_hi_set}");
                println!("is_world_set: {is_world_set}");
            },
        )
        .current_command(
            |FlagHi(is_hi_set), FlagMy(is_my_set), FlagWorld(is_world_set), EmptyTail| {
                println!("is_hi_set: {is_hi_set}");
                println!("is_my_set: {is_my_set}");
                println!("is_world_set: {is_world_set}");
            },
        );
    });
    Ok(())
}

use derive_more::From;

#[derive(From)]
struct FlagHi(bool);
impl utils::FlagBool for FlagHi {
    const NAME: &str = "--hi";
    const ALIASES: &[&str] = &["--hello"];
    const DESCRIPTION: &str = "hello world flag";
}

#[derive(From)]
struct FlagMy(bool);
impl utils::FlagBool for FlagMy {
    const NAME: &str = "--my";
    const DESCRIPTION: &str = "meeee";
}

#[derive(From)]
struct FlagWorld(bool);
impl utils::FlagBool for FlagWorld {
    const NAME: &str = "--world";
    const SHORT_NAME: Option<&str> = Some("-w");
    const DESCRIPTION: &str = "worldldld";
}
