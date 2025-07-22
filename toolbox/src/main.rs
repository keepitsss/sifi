#![warn(clippy::pedantic)]
use std::ffi::OsString;

use anyhow::{Context, Result};

struct Args {
    link_name: OsString,
    to: OsString,
    destination: OsString,
}

impl clap::FromArgMatches for Args {
    fn from_arg_matches(clap_arg_matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut clap_arg_matches.clone())
    }
    fn from_arg_matches_mut(clap_arg_matches: &mut clap::ArgMatches) -> Result<Self, clap::Error> {
        fn get_arg(
            clap_arg_matches: &mut clap::ArgMatches,
            name: &'static str,
        ) -> Result<OsString, clap::error::Error> {
            clap_arg_matches.remove_one(name).ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    format!("The following required argument was not provided: {name}"),
                )
            })
        }
        Ok(Args {
            link_name: get_arg(clap_arg_matches, "link_name")?,
            to: get_arg(clap_arg_matches, "to")?,
            destination: get_arg(clap_arg_matches, "destination")?,
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
        self.set_field(clap_arg_matches, "link_name")?;
        self.set_field(clap_arg_matches, "to")?;
        self.set_field(clap_arg_matches, "destination")?;
        Ok(())
    }
}

impl Args {
    fn set_field(
        &mut self,
        clap_arg_matches: &mut clap::ArgMatches,
        name: &'static str,
    ) -> Result<(), clap::error::Error> {
        if clap_arg_matches.contains_id(name) {
            self.link_name = clap_arg_matches.remove_one(name).ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    format!("The following required argument was not provided: {name}"),
                )
            })?;
        }
        Ok(())
    }
}

impl clap::Args for Args {
    fn group_id() -> Option<clap::Id> {
        Some(clap::Id::from("Args"))
    }
    fn augment_args<'b>(clap_app: clap::Command) -> clap::Command {
        {
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
                        .required(clap::ArgAction::Set.takes_values())
                        .value_parser(clap::value_parser!(OsString))
                        .action(clap::ArgAction::Set),
                )
        }
    }
    fn augment_args_for_update<'b>(clap_app: clap::Command) -> clap::Command {
        {
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
                        .action(clap::ArgAction::Set)
                        .required(false),
                )
                .arg(
                    clap::Arg::new("to")
                        .value_name("TO")
                        .required(clap::ArgAction::Set.takes_values())
                        .value_parser(clap::value_parser!(OsString))
                        .action(clap::ArgAction::Set)
                        .required(false),
                )
                .arg(
                    clap::Arg::new("destination")
                        .value_name("DESTINATION")
                        .required(clap::ArgAction::Set.takes_values())
                        .value_parser(clap::value_parser!(OsString))
                        .action(clap::ArgAction::Set)
                        .required(false),
                )
        }
    }
}

const USAGE: &str = "
{link_name} to {destination}

link_name - path to new link
destination - path to which link_name would be pointing
";

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    match &args[..] {
        [link_name, to, destination] => {
            anyhow::ensure!(to == "to", "{USAGE}");
            std::os::unix::fs::symlink(destination, link_name).with_context(|| {
                format!(
                    "failed to create link named '{}' to destination '{}'",
                    link_name.to_string_lossy(),
                    destination.to_string_lossy()
                )
            })?;
        }
        [] => println!("{USAGE}"),
        [help] if help == "help" || help == "--help" || help == "-h" => println!("{USAGE}"),
        _ => anyhow::bail!("{USAGE}"),
    }

    Ok(())
}
