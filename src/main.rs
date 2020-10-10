#[macro_use]
extern crate lazy_static;
extern crate clap;

use clap::{App, Arg, SubCommand};
use git2::{self, Repository};

mod reflog;
mod submodules;

fn main() {
    let cli_arguments = App::new("Misc git tools")
        .version("0.1")
        .author("Frederik Vestre <freqmod@gmail.com>")
        .about("Extra git porcelain commands")
        .arg(
            Arg::with_name("git_dir")
                .long("git-dir")
                .value_name("GIT_DIR")
                .help("Location of git private files")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("work_dir")
                .long("work-dir")
                .value_name("WORK_DIR")
                .help("Location of checked out data")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("logrefs")
                .about("changes to a branch in the reflog")
                .arg(
                    Arg::with_name("index")
                        .long("index")
                        .short("i")
                        .help("branch index (in list) to change to")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("max_print_refs")
                        .long("max-print-refs")
                        .short("m")
                        .help("maximum number of refs to list")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("remotes")
                        .long("remotes")
                        .short("r")
                        .help("List remotes"),
                )
                .arg(
                    Arg::with_name("tags")
                        .long("tags")
                        .short("t")
                        .help("List tags"),
                ),
        )
        .get_matches();

    let repo = if let Some(_git_dir) = cli_arguments.value_of("git_dir") {
        panic!("Specifying git dir not implemented yet");
    } else {
        Repository::open_from_env().unwrap()
    };

    if let Some(subcmd_arguments) = cli_arguments.subcommand_matches("logrefs") {
        let index = subcmd_arguments
            .value_of("index")
            .and_then(|v| v.parse::<usize>().ok());
        let max_print_refs = subcmd_arguments
            .value_of("max_print_refs")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(20);

        reflog::handle_logrefs(
            repo,
            index,
            subcmd_arguments.is_present("remotes"),
            subcmd_arguments.is_present("tags"),
            max_print_refs,
        )
        .unwrap();
    }
}
