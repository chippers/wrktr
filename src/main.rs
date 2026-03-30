use std::env;

use clap::Parser;
use wrktr::{
    cli::{Cli, Command},
    cmd_clone, cmd_prune, cmd_rm, cmd_worktree,
    error::Error,
};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let git = wrktr::git::backend()?;
    let cwd = env::current_dir()?;

    match cli.command {
        Some(Command::Clone { repo }) => cmd_clone(&git, &repo),
        Some(Command::Prune) => cmd_prune(&git, &cwd),
        Some(Command::Rm { worktree, all }) => cmd_rm(&git, &cwd, worktree.as_deref(), all),
        Some(Command::Linear { issue }) => {
            let key = wrktr::resolve_api_key(cli.linear_api_key.as_deref())?;
            let branch = wrktr::linear::fetch_branch_name(&issue, &key)?;
            println!("{branch}");
            Ok(())
        }
        None => cmd_worktree(
            &git,
            &cwd,
            cli.target.as_deref(),
            cli.issue.as_deref(),
            cli.linear_api_key.as_deref(),
            cli.memory,
        ),
    }
}
