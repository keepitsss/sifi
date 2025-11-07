use super::*;

pub struct Sequence<ITEMS>(pub ITEMS);
macro_rules! impl_sequence {
    ([$(($opt_ty:tt, $opt_name:tt)),+], $last_ty:tt) => {
        impl<$($opt_ty),+> Opt for Sequence<($($opt_ty,)+)>
        where
            $(
            $opt_ty: Opt,
            )+
        {
            fn try_parse_self(
                this: &mut Option<Self>,
                cx: &mut ParsingContext,
            ) -> Result<bool> {
                if this.is_some() {
                    return Ok(false);
                }
                let cursor_checkpoint = cx.cursor;
                $(
                let $opt_name = {
                    let mut value = None;
                    // TODO: ignore error or propagate it?
                    $opt_ty::try_parse_self(&mut value, cx)?;
                    let Ok(value) = $opt_ty::finalize(value) else {
                        cx.cursor = cursor_checkpoint;
                        return Ok(false);
                    };
                    value
                };
                )+
                *this = Some(Sequence(($($opt_name,)+)));
                Ok(true)
            }

            fn finalize(this: Option<Self>) -> Result<Self> {
                if let Some(value) = this {
                    Ok(value)
                } else {
                    Ok(Sequence(($($opt_ty::finalize(None)?,)+)))
                }
            }

            const SECTION: &str = "sequence";

            const DOCUMENTATION: Documentation = Documentation {
                names: Names {
                    main: "some sequence",
                    short: None,
                    aliases: &[],
                },
                description: "Waiting for const_alloc, needed for children docs concatingation"
            };
        }
    };
}
all_the_tuples_named!(impl_sequence);
