use std::{env::ArgsOs, iter::Peekable};

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
    pub possible_flags: Vec<FlagProperies>,
}

pub mod flags;
