use anyhow::Result;
use toolbox::*;

fn main() -> Result<()> {
    parse(main_branch)?;
    // let var = toolbox::start_parsing().flags3((
    //     (("hi",), "hello world flag"),
    //     (("my",), "meeee"),
    //     (("world", 'w'), "worldldld"),
    // ));
    // println!("{var:#?}");
    // let (_hi, _my, _world) = var.flags;
    Ok(())
}
fn main_branch(FlagHi(is_hi_set): FlagHi, FlagMy(is_my_set): FlagMy) {
    dbg!(is_hi_set, is_my_set);
}

fn parse(main_branch: fn(FlagHi, FlagMy)) -> Result<()> {
    let mut cx = ParsingContext::from_args();
    let (mut hi_flag, mut my_flag) = (None, None);
    loop {
        let mut modified = false;
        {
            let parsed = FlagHi::try_parse_self(&mut cx)?;
            if parsed.is_some() {
                anyhow::ensure!(hi_flag.is_none(), "option '--hi' provided twice");
                modified = true;
            }
            hi_flag = hi_flag.or(parsed);
        }
        {
            let parsed = FlagMy::try_parse_self(&mut cx)?;
            if parsed.is_some() {
                anyhow::ensure!(my_flag.is_none(), "option '--my' provided twice");
                modified = true;
            }
            my_flag = my_flag.or(parsed);
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
    let hi_flag = hi_flag
        .ok_or(String::new())
        .or_else(|_| FlagHi::default_case())?;
    let my_flag = my_flag
        .ok_or(String::new())
        .or_else(|_| FlagMy::default_case())?;
    main_branch(hi_flag, my_flag);
    Ok(())
}

struct FlagHi(bool);
impl FlagHi {
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
    const DOCUMENTATION: Documentation = Documentation {
        section: "flag",
        description: "hello world flag",
    };
}

struct FlagMy(bool);
impl FlagMy {
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
        description: "meeee",
    };
}
