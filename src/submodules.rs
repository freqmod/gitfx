#![allow(dead_code)]
#![allow(unused_imports)]

use core::cmp::min;
use git2::{self, Oid, Repository, Signature, SubmoduleIgnore, Time};
use regex::Regex;
use rustyline;

use hex;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};
use std::path::Path;

#[derive(PartialEq)]
enum RunMode {
    All,
    DryOnly,
    WetOnly,
}
fn sync_submodules_inner(
    repo: &Repository,
    dry_run: RunMode,
    force_commit: bool,
) -> Result<(), git2::Error> {
    if dry_run != RunMode::WetOnly {
        for submodule in repo.submodules()? {
            sync_submodules_inner(&submodule.open()?, RunMode::DryOnly, force_commit)?;
            let status =
                repo.submodule_status(submodule.name().unwrap(), SubmoduleIgnore::Unspecified)?;

            /* println!(
                "Submodule {} {:?} {:?} {:?}",
                submodule.name().unwrap(),
                hex::encode(submodule.head_id().unwrap().as_bytes()),
                hex::encode(submodule.index_id().unwrap().as_bytes()),
                hex::encode(submodule.workdir_id().unwrap().as_bytes())
            ); */
            if !status.is_in_index()
                || (!force_commit && (submodule.index_id() != submodule.workdir_id()))
            {
                if status.is_wd_modified() || status.is_wd_wd_modified() {
                    return Err(git2::Error::new(
                        git2::ErrorCode::NotFound,
                        git2::ErrorClass::Reference,
                        "Aborting due to local changes in submodule",
                    ));
                }
            }
        }
    }

    if dry_run == RunMode::DryOnly {
        return Ok(());
    }

    for mut submodule in repo.submodules()? {
        let status =
            repo.submodule_status(submodule.name().unwrap(), SubmoduleIgnore::Unspecified)?;
        if !(!status.is_in_index()
            || (!force_commit && (submodule.index_id() != submodule.workdir_id())))
        {
            assert!(!status.is_wd_wd_modified());
            submodule.update(
                false, None, // TODO: Expose any of these options on the command line?
            )?;
            sync_submodules_inner(&submodule.open()?, RunMode::WetOnly, force_commit)?;
        }
    }
    Ok(())
}

pub fn sync_submodules(repo: &Repository, force_commit: bool) -> Result<(), git2::Error> {
    sync_submodules_inner(repo, RunMode::All, force_commit)
}
