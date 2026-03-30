use std::{cell::RefCell, path::Path};

use wrktr::{cmd_clone, cmd_rm, cmd_worktree, error::Error, git::GitBackend};

// ---------------------------------------------------------------------------
// MockGit
// ---------------------------------------------------------------------------

struct MockGit {
    has_unmerged: bool,
    default_branch: String,
    calls: RefCell<Vec<String>>,
}

impl MockGit {
    fn new() -> Self {
        Self { has_unmerged: false, default_branch: "main".into(), calls: RefCell::new(vec![]) }
    }

    fn with_unmerged(mut self) -> Self {
        self.has_unmerged = true;
        self
    }

    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl GitBackend for MockGit {
    fn clone_repo(&self, url: &str, dest: &Path) -> Result<String, Error> {
        self.calls.borrow_mut().push(format!("clone {} -> {}", url, dest.display()));
        Ok(String::new())
    }

    fn create_branch(&self, _repo: &Path, branch: &str) -> Result<(), Error> {
        self.calls.borrow_mut().push(format!("create_branch {branch}"));
        Ok(())
    }

    fn add_worktree(&self, _repo: &Path, path: &Path, branch: &str) -> Result<(), Error> {
        self.calls.borrow_mut().push(format!("add_worktree {} {}", path.display(), branch));
        Ok(())
    }

    fn remove_worktree(&self, _repo: &Path, path: &Path) -> Result<(), Error> {
        self.calls.borrow_mut().push(format!("remove_worktree {}", path.display()));
        Ok(())
    }

    fn prune_worktrees(&self, _repo: &Path) -> Result<(), Error> {
        self.calls.borrow_mut().push("prune".into());
        Ok(())
    }

    fn has_unmerged_work(
        &self,
        _repo: &Path,
        _worktree_path: &Path,
        _main_branch: &str,
    ) -> Result<bool, Error> {
        Ok(self.has_unmerged)
    }

    fn default_branch(&self, _repo: &Path) -> Result<String, Error> {
        Ok(self.default_branch.clone())
    }
}

// ---------------------------------------------------------------------------
// cmd_clone
// ---------------------------------------------------------------------------

#[test]
fn clone_shorthand_builds_github_url() {
    let git = MockGit::new();
    cmd_clone(&git, "chippers/wrktr").unwrap();

    let calls = git.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0],
        format!(
            "clone https://github.com/chippers/wrktr.git -> {}",
            wrktr::paths::Repo::new("chippers", "wrktr").path().display()
        )
    );
}

#[test]
fn clone_https_url_passes_through() {
    let git = MockGit::new();
    cmd_clone(&git, "https://github.com/chippers/wrktr.git").unwrap();

    let calls = git.calls();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("clone https://github.com/chippers/wrktr.git"));
}

#[test]
fn clone_ssh_url_passes_through() {
    let git = MockGit::new();
    cmd_clone(&git, "git@github.com:chippers/wrktr.git").unwrap();

    let calls = git.calls();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("clone git@github.com:chippers/wrktr.git"));
}

#[test]
fn clone_bare_name_errors() {
    let git = MockGit::new();
    let err = cmd_clone(&git, "wrktr").unwrap_err();
    assert!(err.to_string().contains("expected org/repo or full URL"));
}

// ---------------------------------------------------------------------------
// cmd_rm
// ---------------------------------------------------------------------------

fn repo_cwd() -> std::path::PathBuf {
    wrktr::paths::Repo::new("chippers", "wrktr").path()
}

#[test]
fn rm_with_unmerged_work_errors() {
    let git = MockGit::new().with_unmerged();
    let err = cmd_rm(&git, &repo_cwd(), Some("my-feature"), false).unwrap_err();
    assert!(err.to_string().contains("has unmerged work"));
    assert!(git.calls().is_empty());
}

#[test]
fn rm_without_unmerged_work_removes() {
    let git = MockGit::new();
    cmd_rm(&git, &repo_cwd(), Some("my-feature"), false).unwrap();

    let calls = git.calls();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("remove_worktree"));
    assert!(calls[0].contains("my-feature"));
}

#[test]
fn rm_no_worktree_and_no_all_errors() {
    let git = MockGit::new();
    let err = cmd_rm(&git, &repo_cwd(), None, false).unwrap_err();
    assert!(err.to_string().contains("specify a worktree name"));
}

#[test]
fn rm_outside_managed_repo_errors() {
    let git = MockGit::new();
    let err = cmd_rm(&git, Path::new("/tmp"), Some("branch"), false).unwrap_err();
    assert!(err.to_string().contains("not inside a managed repo"));
}

// ---------------------------------------------------------------------------
// cmd_worktree
// ---------------------------------------------------------------------------

#[test]
fn worktree_plain_branch_creates_and_adds() {
    let git = MockGit::new();
    cmd_worktree(&git, &repo_cwd(), Some("my-feature"), None, None, false).unwrap();

    let calls = git.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0], "create_branch my-feature");
    assert!(calls[1].starts_with("add_worktree"));
    assert!(calls[1].contains("my-feature"));
}

#[test]
fn worktree_no_target_errors() {
    let git = MockGit::new();
    let err = cmd_worktree(&git, &repo_cwd(), None, None, None, false).unwrap_err();
    assert!(err.to_string().contains("specify a branch"));
}

#[test]
fn worktree_outside_managed_repo_errors() {
    let git = MockGit::new();
    let err =
        cmd_worktree(&git, Path::new("/tmp"), Some("branch"), None, None, false).unwrap_err();
    assert!(err.to_string().contains("not inside a managed repo"));
}

// ---------------------------------------------------------------------------
// symlink helpers
// ---------------------------------------------------------------------------

fn unique_tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wrktr-test-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn link_claude_dir_creates_symlink() {
    let tmp = unique_tmp("claude-dir");
    let repo = tmp.join("repo");
    let wt = tmp.join("wt");
    std::fs::create_dir_all(repo.join(".claude")).unwrap();
    std::fs::create_dir_all(&wt).unwrap();

    std::os::unix::fs::symlink(repo.join(".claude"), wt.join(".claude")).unwrap();

    let link = wt.join(".claude");
    assert!(link.exists());
    assert_eq!(std::fs::read_link(&link).unwrap(), repo.join(".claude"));
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn link_claude_memory_creates_symlink() {
    let tmp = unique_tmp("claude-mem");
    let repo = tmp.join("repo");
    let wt = tmp.join("wt");

    let main_memory = wrktr::paths::claude_memory_dir(&repo);
    let wt_project = wrktr::paths::claude_project_dir(&wt);
    std::fs::create_dir_all(&main_memory).unwrap();
    std::fs::create_dir_all(&wt_project).unwrap();
    std::os::unix::fs::symlink(&main_memory, wt_project.join("memory")).unwrap();

    let link = wt_project.join("memory");
    assert!(link.exists());
    assert_eq!(std::fs::read_link(&link).unwrap(), main_memory);
    std::fs::remove_dir_all(&tmp).ok();
}
