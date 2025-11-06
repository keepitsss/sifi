use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail};

struct Executable(PathBuf);
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
            *this = Some(Executable(executable));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn finalize(this: Option<Self>) -> anyhow::Result<Self> {
        this.ok_or(anyhow!("You should provide executable path"))
    }

    const SECTION: &str = "argument";
    const DOCUMENTATION: lib_cli::Documentation = lib_cli::Documentation {
        names: lib_cli::Names::only_main("executable"),
        description: "path to executable",
    };
}

fn main() {
    let dir = format!("{home}/.in_path", home = std::env::var("HOME").unwrap());
    let dir_path = PathBuf::from(dir.clone());
    if let Err(err) = std::fs::create_dir(&dir)
        && err.kind() != ErrorKind::AlreadyExists
    {
        eprintln!("ERROR: Can't create directory '{dir}':\n    {err}");
        return;
    }

    if !std::env::var("PATH")
        .unwrap()
        .split(":")
        .map(|x| x.to_owned())
        .collect::<HashSet<_>>()
        .contains(&dir)
    {
        eprintln!("ERROR: Directory '{dir}' is not in your PATH.");
        return;
    }

    let cx = lib_cli::ParsingContext::from_args(lib_cli::Documentation {
        names: lib_cli::Names {
            main: "executables_installer",
            short: None,
            aliases: &[],
        },
        description: "install static executable in path",
    });
    use lib_cli::ParsingRouter;
    cx.wrapper(|lib_cli::utils::AppPath(_path), lib_cli::TailArgs(args)| {
        args.subcommand(
            lib_cli::Documentation {
                names: lib_cli::Names {
                    main: "install",
                    short: None,
                    aliases: &["add"],
                },
                description: "add executable to your path",
            },
            |Executable(executable), lib_cli::TailArgs(args)| {
                dbg!(executable, args);
                todo!();
            },
        )
        .no_current_command();
    });
    todo!();

    let args = std::env::args().collect::<Vec<_>>();
    let expected_args_count = 1 + (1/*executable*/) + (1/*name*/) + (1/*download_link*/);
    if args.len() != expected_args_count && args.len() != expected_args_count + (1/*description*/) {
        eprintln!(
            "ERROR: Cli interface: %this_app% [executable:path] [name:string] [download_link:url] [description:?string]"
        );
        return;
    }

    let executable = Path::new(&args[1]);
    assert!(executable.exists());
    assert!(executable.is_file());
    let executable = executable.canonicalize().unwrap();
    let name = &args[2];
    assert!(
        name.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    );
    let download_link = &args[3];
    let description = args.get(4);

    assert!(
        String::from_utf8(
            std::process::Command::new("ldd")
                .arg(executable.display().to_string())
                .output()
                .unwrap()
                .stderr,
        )
        .unwrap()
        .contains("not a dynamic executable"),
        "ERROR: Executable must be static."
    );

    let executable_new_location = dir_path.join(name);
    assert!(
        !executable_new_location.exists(),
        "ERROR: Executable with name '{name}' already exists."
    );

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Entry {
        name: String,
        download_link: String,
        description: Option<String>,
    }
    let metadata_path = dir_path.join("metadata.json");
    let metadata_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(metadata_path)
        .unwrap();
    let metadata = std::io::read_to_string(&metadata_file).unwrap();
    let mut metadata = if metadata.is_empty() {
        Vec::new()
    } else {
        let mut metadata: Vec<Entry> = serde_json::from_str(&metadata).unwrap();
        let mut i = 0;
        while i < metadata.len() {
            let entry = &metadata[i];
            if !dir_path.join(&entry.name).exists() {
                metadata.swap_remove(i);
                continue;
            }
            i += 1;
        }
        metadata
    };
    assert!(metadata.iter().all(|e| &e.name != name));
    metadata.push(Entry {
        name: name.to_owned(),
        download_link: download_link.to_owned(),
        description: description.map(|x| x.to_owned()),
    });
    serde_json::to_writer_pretty(metadata_file, &metadata).unwrap();

    std::fs::copy(executable, executable_new_location).unwrap();

    // dbg!(executable, name, download_link, description);
}
