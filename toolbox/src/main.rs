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
fn main_branch(FlagHi(is_hi_set): FlagHi) {
    dbg!(is_hi_set);
}

fn parse(main_branch: fn(FlagHi)) -> Result<()> {
    let mut cx = ParsingContext::from_args();
    let hi_flag = FlagHi::try_parse_self(&mut cx)?;
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
    main_branch(hi_flag);
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
