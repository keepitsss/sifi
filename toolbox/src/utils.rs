use super::*;

pub struct EmptyTail;
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
pub struct TailArgs(pub ParsingContext);
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
    fn try_parse_self(cx: &mut ParsingContext) -> Result<Option<Self>> {
        if let Some(next) = cx.args.get(cx.cursor)
            && let Some(next) = next.to_str()
        {
            if Self::NAME == next
                || Self::SHORT_NAME.is_some_and(|x| x == next)
                || Self::ALIASES.contains(&next)
            {
                cx.cursor += 1;
                Ok(Some(Self::from(true)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn default_case() -> Result<Self> {
        Ok(Self::from(false))
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
