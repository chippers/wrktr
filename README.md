# wrktr

A git worktree manager with Linear integration. Repos live under `~/code/{org}/{repo}` and worktrees under `~/code/worktree/{org}/{repo}/{branch}`.

## Install

```sh
cargo install --git https://github.com/chippers/wrktr
```

## Usage

```
$ wrktr --help
Usage: wrktr [OPTIONS] [TARGET] [COMMAND]

Commands:
  clone   Clone a repo to ~/code/{org}/{repo}
  prune   Prune stale worktree references
  rm      Remove a worktree
  linear  Print the Linear-suggested git branch name for an issue
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [TARGET]  Branch name, org/repo, or Linear issue URL

Options:
  -i, --issue <ISSUE>
          Linear issue ID (e.g. FS-1801)
      --linear-api-key <LINEAR_API_KEY>
          Linear API key (literal, op://vault/item/field, or bw://ItemName). Falls back to
          LINEAR_API_KEY env var [env: LINEAR_API_KEY=]
  -h, --help
          Print help
```

### Clone a repo

```sh
wrktr clone chippers/wrktr
wrktr clone git@github.com:chippers/wrktr.git
```

### Create a worktree

From inside a managed repo (`~/code/{org}/{repo}`):

```sh
# by branch name
wrktr my-feature

# from a Linear issue ID (fetches the suggested branch name)
wrktr -i FS-1801

# from a Linear URL
wrktr https://linear.app/team/issue/FS-1801/some-title
```

The worktree path is printed to stdout, e.g. `~/code/worktree/chippers/wrktr/my-feature`.

### Remove worktrees

```sh
wrktr rm my-feature   # remove one (refuses if there's unmerged work)
wrktr rm --all        # remove all (skips any with unmerged work)
```

### Prune stale references

```sh
wrktr prune
```

## Linear API key

The key is resolved in order:

1. `--linear-api-key` flag
2. `LINEAR_API_KEY` environment variable
3. Auto-discovery from 1Password (`op`) or Bitwarden (`bw`) by searching for items with URL `api.linear.app`

Secret manager references are also supported directly: `op://vault/item/field` or `bw://ItemName`.

## License

Apache-2.0 OR MIT

---

Some code in this project was written with assistance from Claude (Anthropic's AI). A human made all architecture and design decisions, and most of the code.
