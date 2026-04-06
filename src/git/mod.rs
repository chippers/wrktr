mod shell;

use std::path::Path;

pub use shell::ShellGit;

use crate::error::Error;

pub trait GitBackend {
    fn clone_repo(&self, url: &str, dest: &Path) -> Result<String, Error>;
    fn create_branch(&self, repo: &Path, branch: &str) -> Result<(), Error>;
    fn add_worktree(&self, repo: &Path, path: &Path, branch: &str) -> Result<(), Error>;
    fn remove_worktree(&self, repo: &Path, path: &Path) -> Result<(), Error>;
    fn delete_branch(&self, repo: &Path, branch: &str) -> Result<(), Error>;
    fn prune_worktrees(&self, repo: &Path) -> Result<(), Error>;
    fn has_unmerged_work(
        &self,
        repo: &Path,
        worktree_path: &Path,
        main_branch: &str,
    ) -> Result<bool, Error>;
    fn default_branch(&self, repo: &Path) -> Result<String, Error>;
}

pub fn backend() -> Result<ShellGit, Error> {
    ShellGit::new()
}
