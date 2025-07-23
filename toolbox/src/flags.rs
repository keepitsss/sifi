use super::*;

macro_rules! noop {
    ($_t:tt $($sub:tt)+) => {
        $($sub)+
    };
}

macro_rules! add_flags_ext {
    ($name:ident: [$(($param:ident, $i:literal))+]) => {
        pub trait Flags1<F0> {
            fn flags(self, flags: (F0,)) -> ParsingState<(bool,)>;
        }
        impl<F0> Flags1<F0> for ParsingState<()>
        where
            F0: Flag,
        {
            fn flags(mut self, flags: (F0,)) -> ParsingState<(bool,)> {
                let possible_flags = HashSet::from_iter([flags.0.full_properies()]);
                let mut flags = (Some(flags.0.full_properies()),);
                let mut finded = (false,);
                loop {
                    let Some(next_arg) = self.args.peek() else {
                        break;
                    };
                    let Some(next_arg) = next_arg.to_str() else {
                        todo!();
                    };
                    if let Some(ref flag) = flags.0
                        && (flag.long_flag() == next_arg
                            || flag.short_flag().is_some_and(|x| x == next_arg))
                    {
                        finded.0 = true;
                        flags.0 = None;
                        self.args.next().unwrap();
                    }
                    break;
                }
                ParsingState::<(bool,)> {
                    args: self.args,
                    name: self.name,
                    possible_flags,
                    flags: finded,
                }
            }
        }
    };
}
add_flags_ext!(Flags1: [(F0, 0)]);
