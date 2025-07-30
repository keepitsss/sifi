use anyhow::Result;
use toolbox::*;

fn main() -> Result<()> {
    parse(
        |FlagHi(is_hi_set), FlagMy(is_my_set), FlagWorld(is_world_set)| {
            dbg!(is_hi_set, is_my_set, is_world_set);
        },
    )?;
    Ok(())
}

fn parse<T1, T2, T3>(main_branch: impl FnOnce(T1, T2, T3)) -> Result<()>
where
    T1: Opt,
    T2: Opt,
    T3: Opt,
{
    let mut cx = ParsingContext::from_args();
    let (mut opt1, mut opt2, mut opt3) = (None, None, None);
    loop {
        let mut modified = false;
        {
            let parsed = T1::try_parse_self(&mut cx)?;
            if parsed.is_some() {
                anyhow::ensure!(
                    opt1.is_none(),
                    "option '{}' provided twice",
                    T1::ASSOCIATION_NAME
                );
                modified = true;
            }
            opt1 = opt1.or(parsed);
        }
        {
            let parsed = T2::try_parse_self(&mut cx)?;
            if parsed.is_some() {
                anyhow::ensure!(
                    opt2.is_none(),
                    "option '{}' provided twice",
                    T2::ASSOCIATION_NAME
                );
                modified = true;
            }
            opt2 = opt2.or(parsed);
        }
        {
            let parsed = T3::try_parse_self(&mut cx)?;
            if parsed.is_some() {
                anyhow::ensure!(
                    opt3.is_none(),
                    "option '{}' provided twice",
                    T3::ASSOCIATION_NAME
                );
                modified = true;
            }
            opt3 = opt3.or(parsed);
        }
        if !modified {
            break;
        }
    }
    if cx.cursor != cx.args.len() {
        return Err(anyhow::anyhow!(
            "unmatched args: '{}'",
            cx.args[cx.cursor..]
                .iter()
                .map(|x| x.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    let opt1 = opt1.ok_or(String::new()).or_else(|_| T1::default_case())?;
    let opt2 = opt2.ok_or(String::new()).or_else(|_| T2::default_case())?;
    let opt3 = opt3.ok_or(String::new()).or_else(|_| T3::default_case())?;
    main_branch(opt1, opt2, opt3);
    Ok(())
}

trait Opt: Sized {
    fn try_parse_self(cx: &mut ParsingContext) -> Result<Option<Self>>;

    fn default_case() -> Result<Self>;

    const ASSOCIATION_NAME: &str;
    const DOCUMENTATION: Documentation;
}

struct FlagHi(bool);
impl Opt for FlagHi {
    fn try_parse_self(cx: &mut ParsingContext) -> Result<Option<Self>> {
        if let Some(flag) = cx.args.get(cx.cursor)
            && let Some(flag) = flag.to_str()
            && flag.starts_with("--hi")
        {
            cx.cursor += 1;
            if flag != "--hi" {
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

    const ASSOCIATION_NAME: &str = "--hi";
    const DOCUMENTATION: Documentation = Documentation {
        section: "flag",
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

    const ASSOCIATION_NAME: &str = "--my";
    const DOCUMENTATION: Documentation = Documentation {
        section: "flag",
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

    const ASSOCIATION_NAME: &str = "--world";
    const DOCUMENTATION: Documentation = Documentation {
        section: "flag",
        description: "worldldld",
    };
}
