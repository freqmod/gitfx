#![allow(dead_code)]
#![allow(unused_imports)]

use core::cmp::min;
use git2::{self, Oid, Repository, Signature, Time};
use regex::Regex;
use rustyline;

use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};
use std::path::Path;

pub fn sync_submodules(
    repo: Repository,
    index: Option<usize>,
    remotes: bool,
    tags: bool,
    max_print_refs: usize,
) -> Result<(), git2::Error> {

    Ok(())

}