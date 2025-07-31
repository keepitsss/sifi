
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
