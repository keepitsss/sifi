use std::path::PathBuf;

use anyhow::anyhow;

use super::*;

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
impl FinalOpt for TailArgs {
    fn try_parse_self(cx: ParsingContext) -> Result<Self> {
        Ok(Self(cx))
    }
}

pub trait FlagBool: From<bool> {
    const NAME: &str;
    const SHORT_NAME: Option<&str> = None;
    const ALIASES: &[&str] = &[];
    const DESCRIPTION: &str;
}
impl<T> Opt for T
where
    T: FlagBool,
{
    fn try_parse_self(this: &mut Option<Self>, cx: &mut ParsingContext) -> Result<bool> {
        if this.is_some() {
            return Ok(false);
        }
        if let Some(next) = cx.args.get(cx.cursor)
            && let Some(next) = next.to_str()
        {
            if Self::NAME == next
                || Self::SHORT_NAME.is_some_and(|x| x == next)
                || Self::ALIASES.contains(&next)
            {
                cx.cursor += 1;
                *this = Some(Self::from(true));
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn finalize(this: Option<Self>) -> Result<Self> {
        Ok(this.unwrap_or(Self::from(false)))
    }

    const SECTION: &str = "flag";

    const DOCUMENTATION: Documentation = Documentation {
        names: Names {
            main: Self::NAME,
            short: Self::SHORT_NAME,
            aliases: Self::ALIASES,
        },
        description: Self::DESCRIPTION,
    };
}

pub struct FlagHelp(pub bool);
impl From<bool> for FlagHelp {
    fn from(value: bool) -> Self {
        FlagHelp(value)
    }
}
impl utils::FlagBool for FlagHelp {
    const NAME: &str = "--help";
    const SHORT_NAME: Option<&str> = Some("-h");
    const DESCRIPTION: &str = "print help";
}

pub struct AppPath(pub PathBuf);
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
