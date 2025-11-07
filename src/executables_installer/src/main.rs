use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{ErrorKind, Seek, SeekFrom},
    path::PathBuf,
};

mod cli;

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
                dbg!(&executable, &name, &download_link, &comment);
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

                let executable_new_location = dir_path.join(&name);
                assert!(
                    !executable_new_location.exists(),
                    "ERROR: Executable with name '{name}' already exists."
                );

                #[derive(serde::Serialize, serde::Deserialize)]
                struct Entry {
                    name: String,
                    download_link: String,
                    comment: Option<String>,
                }
                let metadata_path = dir_path.join("metadata.json");
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
                assert!(metadata.iter().all(|e| e.name != name));
                metadata.push(Entry {
                    name: name.to_owned(),
                    download_link: download_link.to_owned(),
                    comment: comment.map(|x| x.to_owned()),
                });

                // Overwrite content, not append
                metadata_file.seek(SeekFrom::Start(0)).unwrap();
                metadata_file.set_len(0).unwrap();

                serde_json::to_writer_pretty(metadata_file, &metadata).unwrap();

                std::fs::copy(executable, executable_new_location).unwrap();
            },
        )
        .no_current_command();
    });
}
