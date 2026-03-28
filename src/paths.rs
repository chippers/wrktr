use std::path::{Path, PathBuf};

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").expect("HOME not set"))
}

fn code_dir() -> PathBuf {
    home().join("code")
}

pub fn repo_path(org: &str, repo: &str) -> PathBuf {
    code_dir().join(org).join(repo)
}

pub fn worktree_path(org: &str, repo: &str, branch: &str) -> PathBuf {
    code_dir().join("worktree").join(org).join(repo).join(branch)
}

/// Extract (org, repo) from a path under ~/code/{org}/{repo} or ~/code/worktree/{org}/{repo}/...
pub fn detect_org_repo(cwd: &Path) -> Option<(String, String)> {
    let code = code_dir();
    let rel = cwd.strip_prefix(&code).ok()?;
    let mut parts = rel.components();

    let first = parts.next()?.as_os_str().to_str()?.to_string();

    if first == "worktree" {
        // ~/code/worktree/{org}/{repo}/...
        let org = parts.next()?.as_os_str().to_str()?.to_string();
        let repo = parts.next()?.as_os_str().to_str()?.to_string();
        Some((org, repo))
    } else {
        // ~/code/{org}/{repo}[/...]
        let org = first;
        let repo = parts.next()?.as_os_str().to_str()?.to_string();
        Some((org, repo))
    }
}
