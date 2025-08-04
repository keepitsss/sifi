# Description

Rust library for Command Line Interface, that is small(400 LoC) and simple.

Only depenency is anyhow.

# Library overview

```rust
//! Pseudocode:

Documentation := struct {
    names: struct {
        main: &str,
        short: Option<&str>,
        aliases: &[&str],
    },
    description: &str,
};

/// Has methods:
/// - self.add(Documentation)
/// - self.build() -> actual user-facing documentation
DocumentationStore := struct  {
    item_docs: Documentation,
    store: BTreeMap<(section) &str, (items) Vec<Documentation>>,
};

/// All parsing happens using this struct
ParsingContext := struct {
    args: Vec<OsString>,
    cursor: usize,
    documentation: DocumentationStore,
};

/// Subcommands router
trait ParsingRouter {
    /// tries fall into subcommand with name `docs.names.main`
    fn subcommand(self, docs: Documentation, callback: C) -> Option<ParsingContext>;
    /// fallback
    fn current_command(self, callback: C);
    /// parses args without functionality
    fn wrapper(self, callback: C);
}

/// Actual parsing goes here
pub trait Opt: Sized {
    /// returns whether progess is made
    fn try_parse_self(this: &mut Option<Self>, cx: &mut ParsingContext) -> Result<bool>;

    fn finalize(this: Option<Self>) -> Result<Self>;

    /// For documentaion.
    /// Use 'hidden' to hide.
    const SECTION: &str;
    const DOCUMENTATION: Documentation;
}
```

Also see `src/utils.rs` file.

# Usage

Size of example program is only 102 KB.

```rust
use anyhow::Result;
use lib_cli::*;

#[derive(derive_more::From)]
struct FlagHi(bool);
impl utils::FlagBool for FlagHi {
    const NAME: &str = "--hi";
    const ALIASES: &[&str] = &["--hello"];
    const DESCRIPTION: &str = "hello world flag";
}

#[derive(derive_more::From)]
struct FlagWorld(bool);
impl utils::FlagBool for FlagWorld {
    const NAME: &str = "--world";
    const SHORT_NAME: Option<&str> = Some("-w");
    const DESCRIPTION: &str = "worldldld";
}

fn main() -> Result<()> {
    let cx = ParsingContext::from_args(Documentation {
        names: Names {
            main: "test_program",
            short: None,
            aliases: &[],
        },
        description: "command line parsing library",
    });
    cx.wrapper(|utils::AppPath(_path), utils::TailArgs(args)| {
        args.subcommand(
            Documentation::todo("subcmd"),
            |FlagHi(is_hi_set), utils::EmptyTail| {
                println!("is_hi_set: {is_hi_set}");
            },
        )
        .current_command(
            |FlagHi(is_hi_set), FlagWorld(is_world_set), utils::EmptyTail| {
                println!("is_hi_set: {is_hi_set}");
                println!("is_world_set: {is_world_set}");
            },
        );
    });
    Ok(())
}
```
