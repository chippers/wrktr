pub mod cli;
pub mod error;
pub mod git;
pub mod linear;
pub mod paths;
pub mod secret;

use std::path::Path;

use error::Error;
use git::GitBackend;
use paths::Repo;

pub fn cmd_clone(git: &impl GitBackend, repo: &str) -> Result<(), Error> {
    let (url, repo) = if repo.contains("://") || repo.starts_with("git@") {
        let path_part = repo.trim_end_matches(".git").rsplit('/').collect::<Vec<_>>();
        let name = path_part
            .first()
            .ok_or_else(|| Error::InvalidArgument(format!("cannot parse repo name from: {repo}")))?
            .to_string();
        let org = path_part
            .get(1)
            .ok_or_else(|| Error::InvalidArgument(format!("cannot parse org from: {repo}")))?
            .to_string();
        (repo.to_string(), Repo::new(&org, &name))
    } else {
        let (org, name) = repo.split_once('/').ok_or_else(|| {
            Error::InvalidArgument(format!("expected org/repo or full URL, got: {repo}"))
        })?;
        let url = format!("https://github.com/{org}/{name}.git");
        (url, Repo::new(org, name))
    };

    git.clone_repo(&url, &repo.path())?;
    Ok(())
}

pub fn cmd_prune(git: &impl GitBackend, cwd: &Path) -> Result<(), Error> {
    let repo = Repo::detect(cwd).ok_or_else(|| {
        Error::InvalidArgument("not inside a managed repo (~/code/{org}/{repo})".into())
    })?;
    git.prune_worktrees(&repo.path())
}

pub fn cmd_rm(
    git: &impl GitBackend,
    cwd: &Path,
    worktree: Option<&str>,
    all: bool,
) -> Result<(), Error> {
    let repo = Repo::detect(cwd).ok_or_else(|| {
        Error::InvalidArgument("not inside a managed repo (~/code/{org}/{repo})".into())
    })?;
    let repo_path = repo.path();
    let default_branch = git.default_branch(&repo_path)?;

    if all {
        let worktree_dir = repo.worktree_path("").parent().unwrap().to_path_buf();
        if !worktree_dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(&worktree_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if git.has_unmerged_work(&repo_path, &path, &default_branch)? {
                    eprintln!("skipping {} — has unmerged work", path.display());
                    continue;
                }
                git.remove_worktree(&repo_path, &path)?;
            }
        }
        return Ok(());
    }

    let branch = worktree
        .ok_or_else(|| Error::InvalidArgument("specify a worktree name or use --all".into()))?;
    let wt_path = repo.worktree_path(branch);

    if git.has_unmerged_work(&repo_path, &wt_path, &default_branch)? {
        return Err(Error::InvalidArgument(format!(
            "worktree '{branch}' has unmerged work; use git to clean up first"
        )));
    }
    git.remove_worktree(&repo_path, &wt_path)
}

pub fn cmd_worktree(
    git: &impl GitBackend,
    cwd: &Path,
    target: Option<&str>,
    issue: Option<&str>,
    api_key: Option<&str>,
) -> Result<(), Error> {
    let branch = if let Some(issue_id) = issue {
        let key = resolve_api_key(api_key)?;
        linear::fetch_branch_name(issue_id, &key)?
    } else {
        let target = target.ok_or_else(|| {
            Error::InvalidArgument("specify a branch, issue ID, or Linear URL".into())
        })?;
        if target.contains("linear.app") {
            let issue_id = linear::parse_issue_url(target).ok_or_else(|| {
                Error::InvalidArgument(format!("cannot parse Linear issue ID from: {target}"))
            })?;
            let key = resolve_api_key(api_key)?;
            linear::fetch_branch_name(&issue_id, &key)?
        } else if linear::looks_like_issue_id(target) {
            let key = resolve_api_key(api_key)?;
            linear::fetch_branch_name(target, &key)?
        } else {
            target.to_string()
        }
    };

    let repo = Repo::detect(cwd).ok_or_else(|| {
        Error::InvalidArgument("not inside a managed repo (~/code/{org}/{repo})".into())
    })?;
    let repo_path = repo.path();
    let wt_path = repo.worktree_path(&branch);

    git.create_branch(&repo_path, &branch)?;
    git.add_worktree(&repo_path, &wt_path, &branch)?;

    println!("{}", wt_path.display());
    Ok(())
}

pub fn resolve_api_key(raw: Option<&str>) -> Result<String, Error> {
    match raw {
        Some(raw) => secret::resolve(raw),
        None => secret::discover("api.linear.app"),
    }
}
