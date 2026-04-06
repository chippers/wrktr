use std::{
    path::{Path, PathBuf},
    process::Command,
};

use super::GitBackend;
use crate::error::Error;

pub struct ShellGit {
    git: PathBuf,
}

impl ShellGit {
    pub fn new() -> Result<Self, Error> {
        let git = which::which("git").map_err(|_| Error::Git("git not found in PATH".into()))?;
        Ok(Self { git })
    }

    fn run(&self, repo: &Path, args: &[&str]) -> Result<String, Error> {
        output(Command::new(&self.git).current_dir(repo).args(args))
    }
}

impl GitBackend for ShellGit {
    fn clone_repo(&self, url: &str, dest: &Path) -> Result<String, Error> {
        let parent = dest.parent().ok_or_else(|| Error::Git("invalid dest path".into()))?;
        std::fs::create_dir_all(parent)?;

        output(Command::new(&self.git).current_dir(parent).args([
            "clone",
            url,
            &dest.to_string_lossy(),
        ]))
    }

    fn create_branch(&self, repo: &Path, branch: &str) -> Result<(), Error> {
        self.run(repo, &["branch", branch])?;
        Ok(())
    }

    fn add_worktree(&self, repo: &Path, path: &Path, branch: &str) -> Result<(), Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.run(repo, &["worktree", "add", &path.to_string_lossy(), branch])?;
        Ok(())
    }

    fn remove_worktree(&self, repo: &Path, path: &Path) -> Result<(), Error> {
        self.run(repo, &["worktree", "remove", &path.to_string_lossy()])?;
        Ok(())
    }

    fn delete_branch(&self, repo: &Path, branch: &str) -> Result<(), Error> {
        self.run(repo, &["branch", "-d", branch])?;
        Ok(())
    }

    fn prune_worktrees(&self, repo: &Path) -> Result<(), Error> {
        self.run(repo, &["worktree", "prune"])?;
        Ok(())
    }

    fn has_unmerged_work(
        &self,
        repo: &Path,
        worktree_path: &Path,
        main_branch: &str,
    ) -> Result<bool, Error> {
        let branch = self.run(worktree_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        let range = format!("{main_branch}..{branch}");
        let out = self.run(repo, &["log", &range, "--oneline"])?;
        Ok(!out.is_empty())
    }

    fn default_branch(&self, repo: &Path) -> Result<String, Error> {
        let out = self.run(repo, &["symbolic-ref", "refs/remotes/origin/HEAD"])?;
        // e.g. "refs/remotes/origin/main" -> "main"
        out.rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Git("could not parse default branch".into()))
    }
}

fn output(cmd: &mut Command) -> Result<String, Error> {
    let out = cmd.output()?;
    let trim = |b: &[u8]| String::from_utf8_lossy(b).trim().to_string();
    out.status.success().then(|| trim(&out.stdout)).ok_or_else(|| Error::Git(trim(&out.stderr)))
}
