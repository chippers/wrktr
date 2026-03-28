mod cli;
mod error;
mod git;
mod linear;
mod paths;

use std::env;

use clap::Parser;
use cli::{Cli, Command};
use error::Error;
use git::GitBackend;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let git = git::backend()?;

    match cli.command {
        Some(Command::Clone { repo }) => cmd_clone(&git, &repo),
        Some(Command::Prune) => cmd_prune(&git),
        Some(Command::Rm { worktree, all }) => cmd_rm(&git, worktree.as_deref(), all),
        None => cmd_worktree(&git, cli.target.as_deref(), cli.issue.as_deref()),
    }
}

fn cmd_clone(git: &impl GitBackend, repo: &str) -> Result<(), Error> {
    let (url, org, name) = if repo.contains("://") || repo.starts_with("git@") {
        // Full URL — parse org/repo from the path portion
        let path_part = repo.trim_end_matches(".git").rsplit('/').collect::<Vec<_>>();
        let name = path_part
            .first()
            .ok_or_else(|| Error::InvalidArgument(format!("cannot parse repo name from: {repo}")))?
            .to_string();
        let org = path_part
            .get(1)
            .ok_or_else(|| Error::InvalidArgument(format!("cannot parse org from: {repo}")))?
            .to_string();
        (repo.to_string(), org, name)
    } else {
        // org/repo shorthand
        let (org, name) = repo.split_once('/').ok_or_else(|| {
            Error::InvalidArgument(format!("expected org/repo or full URL, got: {repo}"))
        })?;
        let url = format!("https://github.com/{org}/{name}.git");
        (url, org.to_string(), name.to_string())
    };

    let dest = paths::repo_path(&org, &name);
    git.clone_repo(&url, &dest)?;
    Ok(())
}

fn cmd_prune(git: &impl GitBackend) -> Result<(), Error> {
    let cwd = env::current_dir()?;
    let (org, repo) = paths::detect_org_repo(&cwd).ok_or_else(|| {
        Error::InvalidArgument("not inside a managed repo (~/code/{org}/{repo})".into())
    })?;
    let repo_path = paths::repo_path(&org, &repo);
    git.prune_worktrees(&repo_path)
}

fn cmd_rm(git: &impl GitBackend, worktree: Option<&str>, all: bool) -> Result<(), Error> {
    let cwd = env::current_dir()?;
    let (org, repo) = paths::detect_org_repo(&cwd).ok_or_else(|| {
        Error::InvalidArgument("not inside a managed repo (~/code/{org}/{repo})".into())
    })?;
    let repo_path = paths::repo_path(&org, &repo);
    let default_branch = git.default_branch(&repo_path)?;

    if all {
        let worktree_dir = paths::worktree_path(&org, &repo, "").parent().unwrap().to_path_buf();
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
    let wt_path = paths::worktree_path(&org, &repo, branch);

    if git.has_unmerged_work(&repo_path, &wt_path, &default_branch)? {
        return Err(Error::InvalidArgument(format!(
            "worktree '{branch}' has unmerged work; use git to clean up first"
        )));
    }
    git.remove_worktree(&repo_path, &wt_path)
}

fn cmd_worktree(
    git: &impl GitBackend,
    target: Option<&str>,
    issue: Option<&str>,
) -> Result<(), Error> {
    let target = target.ok_or_else(|| {
        Error::InvalidArgument("specify a branch, issue ID, or Linear URL".into())
    })?;

    // Resolve branch name
    let branch = if let Some(issue_id) = issue {
        linear::fetch_branch_name(issue_id)?
    } else if target.contains("linear.app") {
        let issue_id = linear::parse_issue_url(target).ok_or_else(|| {
            Error::InvalidArgument(format!("cannot parse Linear issue ID from: {target}"))
        })?;
        linear::fetch_branch_name(&issue_id)?
    } else {
        target.to_string()
    };

    // Resolve org/repo
    let cwd = env::current_dir()?;
    let (org, repo) = paths::detect_org_repo(&cwd).ok_or_else(|| {
        Error::InvalidArgument("not inside a managed repo (~/code/{org}/{repo})".into())
    })?;

    let repo_path = paths::repo_path(&org, &repo);
    let wt_path = paths::worktree_path(&org, &repo, &branch);

    git.create_branch(&repo_path, &branch)?;
    git.add_worktree(&repo_path, &wt_path, &branch)?;

    println!("{}", wt_path.display());
    Ok(())
}
