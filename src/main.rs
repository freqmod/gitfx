extern crate clap;
use clap::{App, Arg, SubCommand};
use git2::{self, Repository};
fn handle_reflogbranch(repo: Repository, _index: Option<usize>) -> Result<(), git2::Error> {
    let reflogs = repo.reflog(repo.head()?.name().ok_or(git2::Error::new(
        git2::ErrorCode::NotFound,
        git2::ErrorClass::Reference,
        "Could not convert ref to string",
    ))?);
    for reflog in reflogs.iter() {
        println!("Reflog");
        for entry in reflog.iter() {
            println!("{:?}", entry.message());
        }
    }
    Ok(())
}

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
            SubCommand::with_name("reflogbranch")
                .about("changes to a branch in the reflog")
                .arg(
                    Arg::with_name("index")
                        .short("i")
                        .help("branch index (in list) to change to"),
                ),
        )
        .get_matches();

    let repo = if let Some(_git_dir) = cli_arguments.value_of("git_dir") {
        panic!("Specifying git dir not implemented yet");
    } else {
        Repository::open_from_env().unwrap()
    };

    if let Some(subcmd_arguments) = cli_arguments.subcommand_matches("reflogbranch") {
        let index = subcmd_arguments
            .value_of("index")
            .and_then(|v| v.parse::<usize>().ok());
        handle_reflogbranch(repo, index).unwrap();
    }
}
