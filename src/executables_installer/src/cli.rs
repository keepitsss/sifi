use std::path::PathBuf;

use anyhow::{anyhow, bail};

pub struct Executable(pub PathBuf);
impl lib_cli::Opt for Executable {
    fn try_parse_self(
        this: &mut Option<Self>,
        cx: &mut lib_cli::ParsingContext,
    ) -> anyhow::Result<bool> {
        if this.is_some() {
            return Ok(false);
        }
        if let Some(next) = cx.args.get(cx.cursor)
            && let Some(next) = next.to_str()
        {
            let executable = PathBuf::from(next);
            if !executable.exists() {
                bail!("Executable should exist.");
            }
            if !executable.is_file() {
                bail!("Executable should be file.");
            }
            cx.cursor += 1;
            *this = Some(Executable(executable.canonicalize().unwrap()));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn finalize(this: Option<Self>) -> anyhow::Result<Self> {
        this.ok_or(anyhow!("You should provide executable path"))
    }

    fn add_documentation(store: &mut lib_cli::DocumentationStore) {
        store.add(
            "argument",
            lib_cli::Documentation {
                names: lib_cli::Names::only_main("executable"),
                description: "path to executable",
            },
        );
    }
}

pub struct ExecutableName(pub String);
impl lib_cli::Opt for ExecutableName {
    fn try_parse_self(
        this: &mut Option<Self>,
        cx: &mut lib_cli::ParsingContext,
    ) -> anyhow::Result<bool> {
        if this.is_some() {
            return Ok(false);
        }
        if let Some(next) = cx.args.get(cx.cursor)
            && let Some(next) = next.to_str()
        {
            let name = next;
            if !name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
            {
                bail!("Name could contain alphanumeric characters as well as '_' and '-'.");
            }
            cx.cursor += 1;
            *this = Some(ExecutableName(name.to_owned()));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn finalize(this: Option<Self>) -> anyhow::Result<Self> {
        this.ok_or(anyhow!("You should provide executable name"))
    }

    fn add_documentation(store: &mut lib_cli::DocumentationStore) {
        store.add(
            "argument",
            lib_cli::Documentation {
                names: lib_cli::Names::only_main("name"),
                description: "name of installed executable",
            },
        );
    }
}

pub struct DownloadLink(pub String);
impl lib_cli::Opt for DownloadLink {
    fn try_parse_self(
        this: &mut Option<Self>,
        cx: &mut lib_cli::ParsingContext,
    ) -> anyhow::Result<bool> {
        if this.is_some() {
            return Ok(false);
        }
        if let Some(next) = cx.args.get(cx.cursor)
            && let Some(next) = next.to_str()
        {
            cx.cursor += 1;
            *this = Some(DownloadLink(next.to_owned()));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn finalize(this: Option<Self>) -> anyhow::Result<Self> {
        this.ok_or(anyhow!("You should provide download link"))
    }

    fn add_documentation(store: &mut lib_cli::DocumentationStore) {
        store.add(
            "argument",
            lib_cli::Documentation {
                names: lib_cli::Names::only_main("download_link"),
                description: "where installed executable could be updated or downloaded",
            },
        );
    }
}

pub struct Comment(pub Option<String>);
impl lib_cli::Opt for Comment {
    fn try_parse_self(
        this: &mut Option<Self>,
        cx: &mut lib_cli::ParsingContext,
    ) -> anyhow::Result<bool> {
        if this.is_some() {
            assert!(this.as_ref().unwrap().0.is_some());
            return Ok(false);
        }
        if let Some(next) = cx.args.get(cx.cursor)
            && let Some(next) = next.to_str()
        {
            cx.cursor += 1;
            *this = Some(Comment(Some(next.to_owned())));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn finalize(this: Option<Self>) -> anyhow::Result<Self> {
        Ok(this.unwrap_or(Comment(None)))
    }

    fn add_documentation(store: &mut lib_cli::DocumentationStore) {
        store.add(
            "argument",
            lib_cli::Documentation {
                names: lib_cli::Names::only_main("?comment"),
                description: "any comment, stored in metadata",
            },
        );
    }
}
