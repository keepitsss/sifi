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

#[rustfmt::skip]
macro_rules! all_the_tuples_named {
    ($inner_macro:ident) => {
        $inner_macro!([(T1,n1)], R);
        $inner_macro!([(T1,n1),(T2,n2)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7),(T8,n8)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7),(T8,n8),(T9,n9)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7),(T8,n8),(T9,n9),(T10,n10)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7),(T8,n8),(T9,n9),(T10,n10),(T11,n11)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7),(T8,n8),(T9,n9),(T10,n10),(T11,n11),(T12,n12)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7),(T8,n8),(T9,n9),(T10,n10),(T11,n11),(T12,n12),(T13,n13)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7),(T8,n8),(T9,n9),(T10,n10),(T11,n11),(T12,n12),(T13,n13),(T14,n14)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7),(T8,n8),(T9,n9),(T10,n10),(T11,n11),(T12,n12),(T13,n13),(T14,n14),(T15,n15)], R);
        $inner_macro!([(T1,n1),(T2,n2),(T3,n3),(T4,n4),(T5,n5),(T6,n6),(T7,n7),(T8,n8),(T9,n9),(T10,n10),(T11,n11),(T12,n12),(T13,n13),(T14,n14),(T15,n15),(T16,n16)], R);
    };
}
macro_rules! implement_parsing_callback {
    ([$(($opt_ty:tt, $opt_name:tt)),+], $last_ty:tt) => {
        impl<C, $($opt_ty,)+ $last_ty> ParsingCallback<($($opt_ty),+ , $last_ty)> for C
        where
            C: FnOnce($($opt_ty),+ , $last_ty),
            $(
            $opt_ty: Opt,
            )+
            $last_ty: FinalOpt,
        {
            fn process(mut cx: ParsingContext, callback: Self, add_help: bool) -> Result<()> {
                $(
                cx.documentation.add($opt_ty::SECTION, $opt_ty::DOCUMENTATION);
                )+
                if add_help {
                    cx.documentation
                        .add(FlagHelp::SECTION, FlagHelp::DOCUMENTATION);
                }
                let docs = cx.documentation.build();

                let ($(mut $opt_name,)+) = ($(Option::<$opt_ty>::None,)+);
                loop {
                    let mut modified = false;
                    $(
                        {
                            modified |= $opt_ty::try_parse_self(&mut $opt_name, &mut cx)
                                .map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?;
                        }
                    )+
                    if !modified {
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
                        break;
                    }
                }
                let tail = $last_ty::try_parse_self(cx).map_err(|err| anyhow::anyhow!("{err}\n\n{docs}"))?;
                $(
                let $opt_name = $opt_ty::finalize($opt_name)?;
                )+
                callback($($opt_name),+ , tail);
                Ok(())
            }
        }
    };
}
all_the_tuples_named!(implement_parsing_callback);
