use git2::{self, Oid, Repository, Signature, Time};
pub fn checkout(repo: &Repository, ref_str: &str) -> Result<(), git2::Error> {
    repo.checkout_tree(&repo.revparse_single(ref_str)?, None)?;
    repo.set_head(ref_str)?;
    Ok(())
}
