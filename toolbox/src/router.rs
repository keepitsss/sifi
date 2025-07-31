use super::*;

pub trait ParsingRouter: Sized {
    type Inner;
    fn current_command(self, callback: impl FnOnce(Self::Inner));
    fn subcommand(self, docs: Documentation, callback: impl FnOnce(Self::Inner)) -> Self;
}
impl ParsingRouter for Option<ParsingContext> {
    type Inner = ParsingContext;
    fn current_command(self, callback: impl FnOnce(ParsingContext)) {
        current_command(self, callback)
    }

    fn subcommand(
        self,
        docs: Documentation,
        callback: impl FnOnce(ParsingContext),
    ) -> Option<ParsingContext> {
        subcommand(self, docs, callback)
    }
}

fn current_command(cx: Option<ParsingContext>, callback: impl FnOnce(ParsingContext)) {
    if let Some(cx) = cx {
        callback(cx);
    }
}

fn subcommand(
    cx: Option<ParsingContext>,
    docs: Documentation,
    callback: impl FnOnce(ParsingContext),
) -> Option<ParsingContext> {
    if let Some(mut cx) = cx {
        if let Some(next) = cx.args.get(cx.cursor)
            && let Some(str) = next.to_str()
            && {
                str == docs.names.main
                    || docs.names.short.is_some_and(|x| x == str)
                    || docs.names.aliases.contains(&str)
            }
        {
            cx.cursor += 1;
            cx.documentation = DocumentationStore::new(docs);
            callback(cx);
            None
        } else {
            cx.documentation.add("subcommand", docs);
            Some(cx)
        }
    } else {
        None
    }
}
