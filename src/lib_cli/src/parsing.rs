use std::convert::Infallible;

use super::*;

pub trait ParsingCallback<Inputs = Infallible> {
    fn process(cx: ParsingContext, callback: Self, add_help: bool) -> Result<()>;
}

impl ParsingContext {
    pub fn parse<C, Inputs>(self, callback: C, add_help: bool) -> Result<()>
    where
        C: ParsingCallback<Inputs>,
    {
        C::process(self, callback, add_help)
    }
}

macro_rules! implement_parsing_callback {
    ([$(($opt_ty:tt, $opt_name:tt)),+], $last_ty:tt) => {
        impl<C, $($opt_ty,)+ $last_ty> ParsingCallback<($($opt_ty,)+ $last_ty)> for C
        where
            C: FnOnce($($opt_ty,)+ $last_ty),
            $(
            $opt_ty: Opt,
            )+
            $last_ty: FinalOpt,
        {
            fn process(mut cx: ParsingContext, callback: Self, add_help: bool) -> Result<()> {
                $(
                $opt_ty::add_documentation(&mut cx.documentation);
                )+
                if add_help {
                    FlagHelp::add_documentation(&mut cx.documentation);
                }
                let docs = cx.documentation.build();

                let ($(mut $opt_name,)+) = ($(Option::<$opt_ty>::None,)+);
                loop {
                    if add_help {
                        let mut help_flag = None;
                        FlagHelp::try_parse_self(&mut help_flag, &mut cx)
                            .map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?;
                        if help_flag.is_some_and(|FlagHelp(help_needed)| help_needed)
                        {
                            println!("{}", cx.documentation.build());
                            return Ok(());
                        }
                    }
                    let mut modified = false;
                    $(
                        {
                            modified |= $opt_ty::try_parse_self(&mut $opt_name, &mut cx)
                                .map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?;
                        }
                    )+
                    if !modified {
                        break;
                    }
                }
                let tail = $last_ty::try_parse_self(cx).map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?;
                $(
                let $opt_name = $opt_ty::finalize($opt_name)?;
                )+
                callback($($opt_name,)+ tail);
                Ok(())
            }
        }
    };
}
all_the_tuples_named!(implement_parsing_callback);
