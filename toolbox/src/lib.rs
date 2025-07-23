use std::{collections::HashSet, env::ArgsOs, iter::Peekable};

mod flag;
pub use flag::*;

pub fn start_parsing() -> ParsingState<()> {
    let mut args = std::env::args_os();
    args.next().unwrap();
    ParsingState {
        name: (),
        args: args.peekable(),
        flags: (),
        possible_flags: Default::default(),
    }
}
#[derive(Debug)]
pub struct ParsingState<Flags> {
    pub name: (),
    pub args: Peekable<ArgsOs>,
    pub flags: Flags,
    pub possible_flags: HashSet<FlagProperies>,
}
impl ParsingState<()> {
    pub fn flags(mut self, flags: (impl Flag,)) -> ParsingState<(bool,)> {
        let possible_flags = HashSet::from_iter([flags.0.full_properies()]);
        let mut flags = (Some(flags.0.full_properies()),);
        let mut finded = (false,);
        loop {
            let Some(next_arg) = self.args.peek() else {
                break;
            };
            let Some(next_arg) = next_arg.to_str() else {
                todo!()
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
