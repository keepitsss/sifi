use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{ErrorKind, Seek, SeekFrom},
    ops::Not,
    path::PathBuf,
};

mod cli;

#[derive(serde::Serialize, serde::Deserialize)]
struct MetadataEntry {
    name: String,
    download_link: String,
    comment: Option<String>,
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
            |lib_cli::Sequence((
                cli::Executable(executable),
                cli::ExecutableName(name),
                cli::DownloadLink(download_link),
                cli::Comment(comment),
            )),
             // TODO: collect all(next) args into `comment`
             lib_cli::EmptyTail| {
                assert!(
                    String::from_utf8(
                        std::process::Command::new("readelf")
                            .arg("--program-headers")
                            .arg(executable.display().to_string())
                            .output()
                            .unwrap()
                            .stdout,
                    )
                    .unwrap()
                    .contains("INTERP")
                    .not(),
                    "ERROR: Executable must be static."
                );

                let executable_new_location = dir_path.join(&name);
                assert!(
                    !executable_new_location.exists(),
                    "ERROR: Executable with name '{name}' already exists."
                );

                access_metadata(dir_path.clone(), |metadata| {
                    assert!(metadata.iter().all(|e| e.name != name));
                    metadata.push(MetadataEntry {
                        name: name.to_owned(),
                        download_link: download_link.to_owned(),
                        comment: comment.map(|x| x.to_owned()),
                    });
                });

                std::fs::copy(executable, executable_new_location).unwrap();
            },
        )
        .subcommand(
            lib_cli::Documentation {
                names: lib_cli::Names {
                    main: "uninstall",
                    short: None,
                    aliases: &["delete", "remove"],
                },
                description: "remove installed executable",
            },
            |cli::ExecutableName(executable_name), lib_cli::EmptyTail| {
                let executable = dir_path.clone().join(&executable_name);
                assert!(
                    executable.exists(),
                    "ERROR: Executable with name '{executable_name}' does not exist."
                );
                access_metadata(dir_path.clone(), |metadata| {
                    assert!(
                        metadata.iter().any(|e| e.name == executable_name),
                        "Executable isn't added to metadata"
                    );
                    metadata.retain(|e| e.name != executable_name);
                });

                std::fs::remove_file(executable).unwrap();
            },
        )
        .no_current_command();
    });
}

fn access_metadata(root_dir: PathBuf, callback: impl FnOnce(&mut Vec<MetadataEntry>)) {
    let metadata_path = root_dir.join("metadata.json");
    let mut metadata_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(metadata_path)
        .unwrap();
    let metadata = std::io::read_to_string(&metadata_file).unwrap();
    let mut metadata = if metadata.is_empty() {
        Vec::new()
    } else {
        let mut metadata: Vec<MetadataEntry> = serde_json::from_str(&metadata).unwrap();
        let mut i = 0;
        while i < metadata.len() {
            let entry = &metadata[i];
            if !root_dir.join(&entry.name).exists() {
                metadata.remove(i);
                continue;
            }
            i += 1;
        }
        metadata
    };

    callback(&mut metadata);

    // Overwrite content, not append
    metadata_file.seek(SeekFrom::Start(0)).unwrap();
    metadata_file.set_len(0).unwrap();

    serde_json::to_writer_pretty(metadata_file, &metadata).unwrap();
}
