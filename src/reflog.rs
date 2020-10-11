#[allow(dead_code)]
use core::cmp::min;
use git2::{self, Oid, Repository, Signature, Time};
use regex::Regex;
use rustyline;

use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};
use std::path::Path;

lazy_static! {
    static ref REFLOG_LINE_RE: Regex = Regex::new(concat!(r"^(?P<new>[0-9a-f]{40}) (?P<old>[0-9a-f]{40}) ",
     r"(?P<name>[^<]+) <(?P<email>[^>]+)> (?P<time>[0-9]+) (?P<offset>[\+\-][0-9]{4})\t(?P<message>.*)$")).unwrap();
}

struct ParsedReflogEntry {
    id_new: Oid,
    id_old: Oid,
    committer: Signature<'static>,
    message: String,
}

struct CommitRef {
    reference: String,
    reference_print: String,
    entry: ParsedReflogEntry,
}

fn std_io_err_to_git_err<T>(result: Result<T, std::io::Error>) -> Result<T, git2::Error> {
    result.map_err(|e| {
        git2::Error::new(
            git2::ErrorCode::NotFound,
            git2::ErrorClass::Reference,
            e.to_string(),
        )
    })
}

fn make_reflog_error(msg: Option<String>) -> git2::Error {
    git2::Error::new(
        git2::ErrorCode::Invalid,
        git2::ErrorClass::Reference,
        msg.unwrap_or(String::from("Reflog malformed")),
    )
}

fn parse_reflog_line(
    line_result: Result<String, std::io::Error>,
) -> Result<ParsedReflogEntry, git2::Error> {
    let line_str = std_io_err_to_git_err(line_result)?;
    let capture = REFLOG_LINE_RE
        .captures(line_str.as_str())
        .ok_or(make_reflog_error(Some(String::from(
            "Reflog line not matching regex",
        ))))?;
    Ok(ParsedReflogEntry {
        id_new: Oid::from_str(capture.name("new").ok_or(make_reflog_error(None))?.as_str())?,
        id_old: Oid::from_str(capture.name("old").ok_or(make_reflog_error(None))?.as_str())?,
        committer: Signature::new(
            capture
                .name("name")
                .ok_or(make_reflog_error(None))?
                .as_str(),
            capture
                .name("email")
                .ok_or(make_reflog_error(None))?
                .as_str(),
            &Time::new(
                capture
                    .name("time")
                    .ok_or(make_reflog_error(None))?
                    .as_str()
                    .parse::<i64>()
                    .map_err(|_| make_reflog_error(None))?,
                capture
                    .name("offset")
                    .ok_or(make_reflog_error(None))?
                    .as_str()
                    .parse::<i32>()
                    .map_err(|_| make_reflog_error(None))?,
            ),
        )
        .unwrap(),
        message: capture.name("message").unwrap().as_str().to_string(),
    })
}

type ParseReflogLineFp = fn(
    std::result::Result<std::string::String, std::io::Error>,
) -> std::result::Result<ParsedReflogEntry, git2::Error>;
/* Manually parse reflog, to get info that libgit2 doesn't process */
fn parse_reflog(
    repo: &Repository,
    refname: &str,
) -> Result<
    Option<std::iter::Map<std::io::Lines<std::io::BufReader<std::fs::File>>, ParseReflogLineFp>>,
    git2::Error,
> {
    let gitpath = repo.path();
    let gitrefpath = gitpath.join(Path::new("logs"));
    let refpath = gitrefpath.join(refname);

    match File::open(refpath) {
        Ok(file) => {
            let reader = BufReader::new(file);
            Ok(Some(
                reader.lines().map(parse_reflog_line as ParseReflogLineFp),
            ))
        }
        Err(e) if (e.kind() == ErrorKind::NotFound) => Ok(None),
        Err(e) => std_io_err_to_git_err(Err(e)),
    }
}

fn prompt_for_index(num_refs: usize) -> std::option::Option<usize> {
    let mut rl = rustyline::Editor::<()>::new();
    let readline = rl.readline(">> ");
    match readline {
        Ok(line) => {
            if line.len() == 0 {
                if num_refs != 0 {
                    Some(0)
                } else {
                    None
                }
            } else {
                match line.parse::<usize>() {
                    Ok(index) if index < num_refs => Some(index),
                    Ok(index) => {
                        println!(
                            "Number {} not in range, it has to be less than {}",
                            index, num_refs
                        );
                        None
                    }
                    Err(_) => {
                        println!("Could not parse number");
                        None
                    }
                }
            }
        }
        Err(_) => None,
    }
}

pub fn handle_logrefs(
    repo: Repository,
    index: Option<usize>,
    remotes: bool,
    tags: bool,
    max_print_refs: usize,
) -> Result<(), git2::Error> {
    let mut commitrefs: Vec<CommitRef> = Vec::with_capacity(repo.references()?.names().count());
    let head = repo.head()?;
    let head_name = if !repo.head_detached()? {
        head.name()
    } else {
        None
    };
    for reference_maybe in repo.references()? {
        let reference = reference_maybe?;
        if ((!remotes) && reference.is_remote())
            || ((!tags) && reference.is_tag())
            || (head_name == reference.name())
        {
            continue;
        }
        if let Some(entries) = parse_reflog(
            &repo,
            reference.name().ok_or(git2::Error::new(
                git2::ErrorCode::NotFound,
                git2::ErrorClass::Reference,
                "Could not convert ref to string",
            ))?,
        )? {
            // Only take the last (and most recent) entry
            if let Some(entry_res) = entries.into_iter().last() {
                let entry = entry_res?;
                commitrefs.push(CommitRef {
                    reference: reference.name().unwrap().to_string(),
                    reference_print: reference.shorthand().unwrap().to_string(),
                    entry,
                });
            }
        }
    }

    if head_name.is_some() {
        println!("Current ref: {}", repo.head()?.shorthand().unwrap());
    }

    commitrefs.sort_by_key(|v| v.entry.committer.when());
    commitrefs.reverse();

    for commit_idx in 0..min(max_print_refs, commitrefs.len()) {
        let commitref = commitrefs.get(commit_idx).unwrap();
        println!(
            "{:03} {}: {}",
            commit_idx, commitref.reference_print, commitref.entry.message,
        );
    }

    let new_index_maybe = match index {
        Some(index) => Some(index),
        None => prompt_for_index(commitrefs.len()),
    };

    if let Some(new_index) = new_index_maybe {
        let ref_str = commitrefs.get(new_index).unwrap().reference.as_str();
        repo.checkout_tree(&repo.revparse_single(ref_str)?, None)?;
        repo.set_head(ref_str)?;
    }
    Ok(())
}
