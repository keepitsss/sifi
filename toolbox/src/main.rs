use std::ffi::OsString;

use anyhow::Result;
use toolbox::*;

fn main() -> Result<()> {
    parse(
        |FlagHi(is_hi_set), FlagMy(is_my_set), FlagWorld(is_world_set), TailArgs(tail)| {
            dbg!(is_hi_set, is_my_set, is_world_set, tail);
        },
    )?;
    Ok(())
}

fn parse<T1, T2, T3, R>(main_branch: impl FnOnce(T1, T2, T3, R)) -> Result<()>
where
    T1: Opt,
    T2: Opt,
    T3: Opt,
    R: FinalOpt,
{
    let mut cx = ParsingContext::from_args();
    cx.documentation.add(T1::DOCUMENTATION);
    cx.documentation.add(T2::DOCUMENTATION);
    cx.documentation.add(T3::DOCUMENTATION);
    cx.documentation.add(FlagHelp::DOCUMENTATION);
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

struct EmptyTail;
impl FinalOpt for EmptyTail {
    fn try_parse_self(cx: ParsingContext) -> Result<Self> {
        if cx.cursor == cx.args.len() {
            Ok(EmptyTail)
        } else {
            Err(anyhow::anyhow!(
                "unmatched args: '{}'",
                cx.args[cx.cursor..]
                    .iter()
                    .map(|x| x.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" ")
            ))
        }
    }
}
struct TailArgs(Vec<OsString>);
impl FinalOpt for TailArgs {
    fn try_parse_self(cx: ParsingContext) -> Result<Self> {
        Ok(Self(
            cx.args
                .get(cx.cursor..)
                .map(|x| x.to_owned())
                .unwrap_or_default(),
        ))
    }
}

struct FlagHelp(bool);
impl Opt for FlagHelp {
    fn try_parse_self(cx: &mut ParsingContext) -> Result<Option<Self>> {
        if let Some(flag) = cx.args.get(cx.cursor)
            && let Some(flag) = flag.to_str()
        {
            if flag.starts_with("--help") {
                cx.cursor += 1;
                anyhow::ensure!(
                    flag == "--help",
                    "flag '{flag}' not fount. maybe you mean '--help'"
                );
                Ok(Some(FlagHelp(true)))
            } else if flag.starts_with("-h") {
                cx.cursor += 1;
                anyhow::ensure!(
                    flag == "-h",
                    "short flag '{flag}' not fount. maybe you mean '-h(--help)'"
                );
                Ok(Some(FlagHelp(true)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn default_case() -> Result<Self> {
        Ok(FlagHelp(false))
    }

    const DOCUMENTATION: Documentation = Documentation {
        section: "flag",
        names: OptNames {
            main: "--help",
            short: Some("-h"),
            aliases: &[],
        },
        description: "print help",
    };
}

struct FlagHi(bool);
impl Opt for FlagHi {
    fn try_parse_self(cx: &mut ParsingContext) -> Result<Option<Self>> {
        if let Some(flag) = cx.args.get(cx.cursor)
            && let Some(flag) = flag.to_str()
            && (flag.starts_with("--hi") || flag.starts_with("--hello"))
        {
            cx.cursor += 1;
            if flag != "--hi" && flag != "--hello" {
                return Err(anyhow::anyhow!(
                    "flag '{flag}' not fount. maybe you mean '--hi'"
                ));
            }
            Ok(Some(FlagHi(true)))
        } else {
            Ok(None)
        }
    }

    fn default_case() -> Result<Self> {
        Ok(FlagHi(false))
    }

    const DOCUMENTATION: Documentation = Documentation {
        section: "flag",
        names: OptNames {
            main: "--hi",
            short: None,
            aliases: &["--hello"],
        },
        description: "hello world flag",
    };
}

struct FlagMy(bool);
impl Opt for FlagMy {
    fn try_parse_self(cx: &mut ParsingContext) -> Result<Option<Self>> {
        if let Some(flag) = cx.args.get(cx.cursor)
            && let Some(flag) = flag.to_str()
            && flag.starts_with("--my")
        {
            cx.cursor += 1;
            if flag != "--my" {
                return Err(anyhow::anyhow!(
                    "flag '{flag}' not fount. maybe you mean '--my'"
                ));
            }
            Ok(Some(FlagMy(true)))
        } else {
            Ok(None)
        }
    }

    fn default_case() -> Result<Self> {
        Ok(FlagMy(false))
    }

    const DOCUMENTATION: Documentation = Documentation {
        section: "flag",
        names: OptNames {
            main: "--my",
            short: None,
            aliases: &[],
        },
        description: "meeee",
    };
}

struct FlagWorld(bool);
impl Opt for FlagWorld {
    fn try_parse_self(cx: &mut ParsingContext) -> Result<Option<Self>> {
        if let Some(flag) = cx.args.get(cx.cursor)
            && let Some(flag) = flag.to_str()
        {
            if flag.starts_with("--world") {
                cx.cursor += 1;
                anyhow::ensure!(
                    flag == "--world",
                    "flag '{flag}' not fount. maybe you mean '--world'"
                );
                Ok(Some(FlagWorld(true)))
            } else if flag.starts_with("-w") {
                cx.cursor += 1;
                anyhow::ensure!(
                    flag == "-w",
                    "short flag '{flag}' not fount. maybe you mean '-w(--world)'"
                );
                Ok(Some(FlagWorld(true)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn default_case() -> Result<Self> {
        Ok(FlagWorld(false))
    }

    const DOCUMENTATION: Documentation = Documentation {
        section: "flag",
        names: OptNames {
            main: "--world",
            short: Some("-w"),
            aliases: &[],
        },
        description: "worldldld",
    };
}
