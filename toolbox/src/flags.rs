use super::*;

macro_rules! noop {
    ($_t:tt $($sub:tt)+) => {
        $($sub)+
    };
}

macro_rules! add_flags_methods {
    ($method_name:ident: [$(($param:ident, $i:tt)),+]) => {
        impl ParsingState<()>
        {
            pub fn $method_name<$($param),+>(mut self, flags: ($($param,)+) ) -> ParsingState<($(noop!(($param) bool),)+)>
            where
                $($param: Flag),+
            {
                let possible_flags = Vec::from([$(flags.$i.full_properies()),+]);
                let flags = ($((flags.$i.full_properies()),)+);
                let mut finded = ($(noop!(($i) false),)+);
                loop {
                    let Some(next_arg) = self.args.peek() else {
                        break;
                    };
                    let Some(next_arg) = next_arg.to_str() else {
                        todo!();
                    };
                    $(
                    if flags.$i.long_flag() == next_arg
                       || flags.$i.short_flag().is_some_and(|x| x == next_arg)
                    {
                        if finded.$i {
                            todo!("duplicate");
                        }
                        finded.$i = true;
                        self.args.next().unwrap();
                        continue;
                    }
                    )+
                    break;
                }
                ParsingState::<($(noop!(($param) bool),)+)> {
                    args: self.args,
                    name: self.name,
                    possible_flags,
                    flags: finded,
                }
            }
        }
    };
}

add_flags_methods!(flags1: [(F0, 0)]);
add_flags_methods!(flags2: [(F0, 0), (F1, 1)]);
add_flags_methods!(flags3: [(F0, 0), (F1, 1), (F2, 2)]);
add_flags_methods!(flags4: [(F0, 0), (F1, 1), (F2, 2), (F3, 3)]);
