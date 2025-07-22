#![warn(clippy::pedantic)]
use std::{ffi::OsString, path::Path};

use anyhow::Result;
use clap::{Args, CommandFactory, FromArgMatches};

struct MyArgs {
    link_name: OsString,
    to: OsString,
    destination: OsString,
}
impl clap::Parser for MyArgs {}

impl clap::CommandFactory for MyArgs {
    fn command<'b>() -> clap::Command {
        <Self as clap::Args>::augment_args(clap::Command::new("toolbox"))
    }
    fn command_for_update<'b>() -> clap::Command {
        <Self as clap::Args>::augment_args_for_update(clap::Command::new("toolbox"))
    }
}
impl clap::FromArgMatches for MyArgs {
    fn from_arg_matches(clap_arg_matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut clap_arg_matches.clone())
    }
    fn from_arg_matches_mut(clap_arg_matches: &mut clap::ArgMatches) -> Result<Self, clap::Error> {
        Ok(MyArgs {
            link_name: clap_arg_matches.remove_one("link_name").ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "The following required argument was not provided: link_name",
                )
            })?,
            to: clap_arg_matches.remove_one("to").ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "The following required argument was not provided: to",
                )
            })?,
            destination: clap_arg_matches.remove_one("destination").ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "The following required argument was not provided: destination",
                )
            })?,
        })
    }
    fn update_from_arg_matches(
        &mut self,
        clap_arg_matches: &clap::ArgMatches,
    ) -> Result<(), clap::Error> {
        self.update_from_arg_matches_mut(&mut clap_arg_matches.clone())
    }
    fn update_from_arg_matches_mut(
        &mut self,
        clap_arg_matches: &mut clap::ArgMatches,
    ) -> Result<(), clap::Error> {
        if clap_arg_matches.contains_id("link_name") {
            self.link_name = clap_arg_matches.remove_one("link_name").ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "The following required argument was not provided: link_name",
                )
            })?;
        }
        if clap_arg_matches.contains_id("to") {
            self.to = clap_arg_matches.remove_one("to").ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "The following required argument was not provided: to",
                )
            })?;
        }
        if clap_arg_matches.contains_id("destination") {
            self.destination = clap_arg_matches.remove_one("destination").ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "The following required argument was not provided: destination",
                )
            })?;
        }
        Result::Ok(())
    }
}
impl clap::Args for MyArgs {
    fn group_id() -> Option<clap::Id> {
        Some(clap::Id::from("Args"))
    }
    fn augment_args<'b>(clap_app: clap::Command) -> clap::Command {
        clap_app
            .group(clap::ArgGroup::new("Args").multiple(true).args([
                clap::Id::from("link_name"),
                clap::Id::from("to"),
                clap::Id::from("destination"),
            ]))
            .arg(
                clap::Arg::new("link_name")
                    .value_name("LINK_NAME")
                    .required(clap::ArgAction::Set.takes_values())
                    .value_parser(clap::value_parser!(OsString))
                    .action(clap::ArgAction::Set),
            )
            .arg(
                clap::Arg::new("to")
                    .value_name("TO")
                    .required(clap::ArgAction::Set.takes_values())
                    .value_parser(clap::value_parser!(OsString))
                    .action(clap::ArgAction::Set),
            )
            .arg(
                clap::Arg::new("destination")
                    .value_name("DESTINATION")
                    .required(true && clap::ArgAction::Set.takes_values())
                    .value_parser(clap::value_parser!(OsString))
                    .action(clap::ArgAction::Set),
            )
    }
    fn augment_args_for_update<'b>(clap_app: clap::Command) -> clap::Command {
        clap_app
            .group(clap::ArgGroup::new("Args").multiple(true).args([
                clap::Id::from("link_name"),
                clap::Id::from("to"),
                clap::Id::from("destination"),
            ]))
            .arg(
                clap::Arg::new("link_name")
                    .value_name("LINK_NAME")
                    .required(true && clap::ArgAction::Set.takes_values())
                    .value_parser(clap::value_parser!(OsString))
                    .action(clap::ArgAction::Set)
                    .required(false),
            )
            .arg(
                clap::Arg::new("to")
                    .value_name("TO")
                    .required(true && clap::ArgAction::Set.takes_values())
                    .value_parser(clap::value_parser!(OsString))
                    .action(clap::ArgAction::Set)
                    .required(false),
            )
            .arg(
                clap::Arg::new("destination")
                    .value_name("DESTINATION")
                    .required(true && clap::ArgAction::Set.takes_values())
                    .value_parser(clap::value_parser!(OsString))
                    .action(clap::ArgAction::Set)
                    .required(false),
            )
    }
}

const USAGE: &str = "
{link_name} to {destination}

link_name - path to new link
destination - path to which link_name would be pointing
";

fn main() -> Result<()> {
    let args = {
        match MyArgs::from_arg_matches_mut(&mut {
            {
                let mut this = clap::Command::new("toolbox")
                    .group(clap::ArgGroup::new("Args").multiple(true).args([
                        clap::Id::from("link_name"),
                        clap::Id::from("to"),
                        clap::Id::from("destination"),
                    ]))
                    .arg(
                        clap::Arg::new("link_name")
                            .value_name("LINK_NAME")
                            .required(clap::ArgAction::Set.takes_values())
                            .value_parser(clap::value_parser!(OsString))
                            .action(clap::ArgAction::Set),
                    )
                    .arg(
                        clap::Arg::new("to")
                            .value_name("TO")
                            .required(clap::ArgAction::Set.takes_values())
                            .value_parser(clap::value_parser!(OsString))
                            .action(clap::ArgAction::Set),
                    )
                    .arg(
                        clap::Arg::new("destination")
                            .value_name("DESTINATION")
                            .required(clap::ArgAction::Set.takes_values())
                            .value_parser(clap::value_parser!(OsString))
                            .action(clap::ArgAction::Set),
                    );

                let this = &mut this;
                let mut raw_args = clap_lex::RawArgs::new(std::env::args_os());
                let mut cursor = raw_args.cursor();

                // Get the name of the program (argument 1 of env::args()) and determine the
                // actual file
                // that was used to execute the program. This is because a program called
                // ./target/release/my_prog -a
                // will have two arguments, './target/release/my_prog', '-a' but we don't want
                // to display
                // the full path when displaying help messages and such
                if let Some(name) = raw_args.next_os(&mut cursor) {
                    let p = Path::new(name);

                    if let Some(f) = p.file_name()
                        && let Some(s) = f.to_str()
                        && this.get_bin_name().is_none()
                    {
                        this.bin_name(s.to_owned());
                    }
                }

                let this = &mut *this;
                let raw_args: &mut clap_lex::RawArgs = &mut raw_args;
                debug!("Command::_do_parse");

                // // If there are global arguments, or settings we need to propagate them down to subcommands
                // // before parsing in case we run into a subcommand
                // this._build_self(false);

                let mut matcher = clap_builder::parser::arg_matcher::ArgMatcher::new(this);

                // do the real parsing
                let mut parser = clap_builder::parser::parser::Parser::new(this);
                parser
                    .get_matches_with(&mut matcher, raw_args, cursor)
                    .unwrap_or_else(|e| {
                        drop(this);
                        e.exit()
                    });
                let mut global_arg_vec = Default::default();
                this.get_used_global_args(&matcher, &mut global_arg_vec);

                matcher.propagate_globals(&global_arg_vec);

                matcher.into_inner()
            }
        })
        .map_err(|x| x.format(&mut MyArgs::command()))
        {
            Ok(s) => s,
            Err(e) => {
                // Since this is more of a development-time error, we aren't doing as fancy of a quit
                // as `get_matches`
                e.exit()
            }
        }
    };
    // let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    // match &args[..] {
    //     [link_name, to, destination] => {
    //         anyhow::ensure!(to == "to", "{USAGE}");
    //         std::os::unix::fs::symlink(destination, link_name).with_context(|| {
    //             format!(
    //                 "failed to create link named '{}' to destination '{}'",
    //                 link_name.to_string_lossy(),
    //                 destination.to_string_lossy()
    //             )
    //         })?;
    //     }
    //     [] => println!("{USAGE}"),
    //     [help] if help == "help" || help == "--help" || help == "-h" => println!("{USAGE}"),
    //     _ => anyhow::bail!("{USAGE}"),
    // }

    Ok(())
}
