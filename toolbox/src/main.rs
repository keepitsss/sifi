use std::path::PathBuf;

use anyhow::{Result, anyhow};
use toolbox::*;

fn main() -> Result<()> {
    let cx = ParsingContext::from_args(Documentation {
        names: Names {
            main: "test_program",
            short: None,
            aliases: &[],
        },
        description: "command line parsing library",
    });
    Some(cx).wrapper(|AppPath(_path), utils::TailArgs(args)| {
        Some(args)
            .subcommand(
                Documentation::todo("subcmd"),
                |FlagHi(is_hi_set), FlagWorld(is_world_set), utils::EmptyTail| {
                    dbg!(is_hi_set, is_world_set);
                },
            )
            .current_command(
                |FlagHi(is_hi_set),
                 FlagMy(is_my_set),
                 FlagWorld(is_world_set),
                 utils::EmptyTail| {
                    dbg!(is_hi_set, is_my_set, is_world_set);
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

struct AppPath(PathBuf);
impl Opt for AppPath {
    fn try_parse_self(this: &mut Option<Self>, cx: &mut ParsingContext) -> Result<bool> {
        if this.is_some() {
            return Ok(false);
        }
        if cx.cursor != 0 {
            return Err(anyhow!("App Path should go first"));
        }
        let name = cx.args.first();
        if let Some(name) = name {
            *this = Some(AppPath(PathBuf::from(name)));
            cx.cursor += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn finalize(this: Option<Self>) -> Result<Self> {
        this.ok_or(anyhow!("First argument should be App Path"))
    }

    const SECTION: &str = "hidden";

    const DOCUMENTATION: Documentation = Documentation {
        names: Names {
            main: "APP_PATH",
            short: None,
            aliases: &[],
        },
        description: "path to this program",
    };
}
