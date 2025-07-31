use anyhow::Result;
use toolbox::*;

fn main() -> Result<()> {
    let mut cx = ParsingContext::from_args(Documentation {
        names: Names {
            main: "test_program",
            short: None,
            aliases: &[],
        },
        description: "command line parsing library",
    });
    cx.cursor += 1;
    Some(cx)
        .subcommand(Documentation::todo("subcmd"), |cx| {
            parse(
                cx,
                |FlagHi(is_hi_set),
                 FlagMy(is_my_set),
                 FlagWorld(is_world_set),
                 utils::TailArgs(tail)| {
                    dbg!(is_hi_set, is_my_set, is_world_set, tail);
                },
            )
            .unwrap();
        })
        .current_command(|cx| {
            parse(
                cx,
                |FlagHi(is_hi_set),
                 FlagMy(is_my_set),
                 FlagWorld(is_world_set),
                 utils::TailArgs(tail)| {
                    dbg!(is_hi_set, is_my_set, is_world_set, tail);
                },
            )
            .unwrap();
        });
    Ok(())
}

fn parse<T1, T2, T3, R>(
    mut cx: ParsingContext,
    main_branch: impl FnOnce(T1, T2, T3, R),
) -> Result<()>
where
    T1: Opt,
    T2: Opt,
    T3: Opt,
    R: FinalOpt,
{
    cx.documentation.add(T1::SECTION, T1::DOCUMENTATION);
    cx.documentation.add(T2::SECTION, T2::DOCUMENTATION);
    cx.documentation.add(T3::SECTION, T3::DOCUMENTATION);
    cx.documentation
        .add(FlagHelp::SECTION, FlagHelp::DOCUMENTATION);
    let docs = cx.documentation.build();

    let (mut opt1, mut opt2, mut opt3) = (None, None, None);
    loop {
        let mut modified = false;
        {
            let parsed =
                T1::try_parse_self(&mut cx).map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?;
            if parsed.is_some() {
                anyhow::ensure!(
                    opt1.is_none(),
                    "option '{}' provided twice",
                    T1::DOCUMENTATION.names.main
                );
                modified = true;
            }
            opt1 = opt1.or(parsed);
        }
        {
            let parsed =
                T2::try_parse_self(&mut cx).map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?;
            if parsed.is_some() {
                anyhow::ensure!(
                    opt2.is_none(),
                    "option '{}' provided twice",
                    T2::DOCUMENTATION.names.main
                );
                modified = true;
            }
            opt2 = opt2.or(parsed);
        }
        {
            let parsed =
                T3::try_parse_self(&mut cx).map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?;
            if parsed.is_some() {
                anyhow::ensure!(
                    opt3.is_none(),
                    "option '{}' provided twice",
                    T3::DOCUMENTATION.names.main
                );
                modified = true;
            }
            opt3 = opt3.or(parsed);
        }
        if !modified {
            if FlagHelp::try_parse_self(&mut cx)
                .map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?
                .is_some_and(|FlagHelp(help_needed)| help_needed)
            {
                println!("{}", cx.documentation.build());
                return Ok(());
            }
            break;
        }
    }
    let tail = R::try_parse_self(cx).map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?;
    let opt1 = opt1.ok_or(String::new()).or_else(|_| T1::default_case())?;
    let opt2 = opt2.ok_or(String::new()).or_else(|_| T2::default_case())?;
    let opt3 = opt3.ok_or(String::new()).or_else(|_| T3::default_case())?;
    main_branch(opt1, opt2, opt3, tail);
    Ok(())
}

use derive_more::From;

#[derive(From)]
struct FlagHelp(bool);
impl utils::FlagBool for FlagHelp {
    const NAME: &str = "--help";
    const SHORT_NAME: Option<&str> = Some("-h");
    const DESCRIPTION: &str = "print help";
}

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
