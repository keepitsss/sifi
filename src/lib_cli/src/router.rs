use super::*;

pub trait ParsingRouter {
    fn subcommand<C, Inputs>(self, docs: Documentation, callback: C) -> Option<ParsingContext>
    where
        C: ParsingCallback<Inputs>;
    fn current_command<C, Inputs>(self, callback: C)
    where
        C: ParsingCallback<Inputs>;
    fn wrapper<C, Inputs>(self, callback: C)
    where
        C: ParsingCallback<Inputs>;
}
impl ParsingRouter for Option<ParsingContext> {
    fn subcommand<C, Inputs>(self, docs: Documentation, callback: C) -> Option<ParsingContext>
    where
        C: ParsingCallback<Inputs>,
    {
        subcommand(self, docs, |cx| {
            cx.parse(callback, true).unwrap();
        })
    }
    fn current_command<C, Inputs>(self, callback: C)
    where
        C: ParsingCallback<Inputs>,
    {
        current_command(self, |cx| {
            cx.parse(callback, true).unwrap();
        })
    }
    fn wrapper<C, Inputs>(self, callback: C)
    where
        C: ParsingCallback<Inputs>,
    {
        current_command(self, |cx| {
            cx.parse(callback, false).unwrap();
        })
    }
}
impl ParsingRouter for ParsingContext {
    fn subcommand<C, Inputs>(self, docs: Documentation, callback: C) -> Option<ParsingContext>
    where
        C: ParsingCallback<Inputs>,
    {
        Some(self).subcommand(docs, callback)
    }

    fn current_command<C, Inputs>(self, callback: C)
    where
        C: ParsingCallback<Inputs>,
    {
        Some(self).current_command(callback);
    }

    fn wrapper<C, Inputs>(self, callback: C)
    where
        C: ParsingCallback<Inputs>,
    {
        Some(self).wrapper(callback);
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
