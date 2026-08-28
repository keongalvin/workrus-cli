# workrus

`workrus` is a small, independently implemented Rust CLI for [Linear](https://linear.app) workflows. It is inspired by [`schpet/linear-cli`](https://github.com/schpet/linear-cli); no upstream source code was copied.

## Requirements and setup

Rust 1.93 and Git are required. Install a published release with `cargo install workrus` or from a checkout with `cargo install --path .`. Export a Linear personal API key; it is read only from the environment and is never stored:

```sh
export LINEAR_API_KEY='…'
workrus config ENG
```

Configuration is global only: Unix uses `${XDG_CONFIG_HOME:-$HOME/.config}/linear/linear.toml`; Windows uses `%APPDATA%\linear\linear.toml`. `config TEAM` validates the team remotely before writing `team_id` and does not require a Git worktree. Environment values override TOML: `LINEAR_TEAM_ID`, `LINEAR_WORKSPACE`, `LINEAR_ISSUE_SORT` (`manual` or `priority`), `LINEAR_ISSUE_CREATE_ASK_PROJECT` (`true` or `false`), and `LINEAR_ISSUE_CREATE_ASSIGN_SELF` (`always`, `auto`, or `never`). `LINEAR_API_KEY` remains environment-only. Repository `.workrus` and local `linear.toml` files are never read.

## Commands

```text
workrus config [TEAM]
workrus completion|completions <bash|zsh|fish|powershell>
workrus team list [--limit N] [--after CURSOR] [--web|--app]
workrus team id [TEAM]
workrus team members [TEAM] [--all] [--limit N] [--after CURSOR]
workrus team create --name NAME [--key KEY] [--description TEXT] [--color COLOR] [--icon ICON]
workrus team autolinks
workrus user list [--all] [--limit N] [--after CURSOR]
workrus project list [--team TEAM] [--limit N] [--after CURSOR] [--web|--app]
workrus project view PROJECT [--web|--app]
workrus project create --name NAME --team TEAM [--team TEAM...] [--description TEXT] [--content-file PATH] [--lead USER] [--member USER...] [--target-date YYYY-MM-DD]
workrus milestone|m list --project PROJECT [--limit N] [--after CURSOR]
workrus milestone|m view MILESTONE
workrus milestone|m create --project PROJECT --name NAME [--description TEXT] [--target-date YYYY-MM-DD]
workrus milestone|m update MILESTONE [--name NAME] [--description TEXT] [--target-date YYYY-MM-DD]
workrus milestone|m delete MILESTONE --confirm MILESTONE [--dry-run]
workrus document|docs list [--project PROJECT|--issue ISSUE] [--limit N] [--after CURSOR]
workrus document|docs view DOCUMENT [--raw] [-w|--web]
workrus document|docs create --title TITLE (--project PROJECT|--issue ISSUE) [--content-file PATH|--stdin]
workrus document|docs update DOCUMENT [--title TITLE] [--content-file PATH|--stdin] [--project PROJECT|--issue ISSUE] [--force]
workrus document|docs delete DOCUMENT --confirm DOCUMENT [--dry-run]
workrus document|docs delete --bulk DOCUMENT... --confirm DOCUMENT... [--dry-run]
workrus issue mine|list|l [--team TEAM] [-s STATE] [--sort manual|priority] [--project PROJECT] [--milestone MILESTONE] [--limit N] [--after CURSOR] [--web|--app]
workrus issue query <TEXT|--search TEXT> [--team TEAM|--all-teams] [-s STATE] [--sort manual|priority] [--project PROJECT] [--milestone MILESTONE] [--limit N] [--after CURSOR]
workrus issue view [ID|NUMBER] [--web|--app]
workrus issue pr|pull-request [ID|NUMBER] [--base BRANCH] [--head BRANCH] [--draft] [-t TITLE] [-w|--web]
workrus issue create [-t TITLE] [-d DESCRIPTION] [--team TEAM] [--assignee self] [--state STATE] [--project PROJECT] [--milestone MILESTONE] [--priority 0..4]
workrus issue update [ID|NUMBER] [-t TITLE] [-d DESCRIPTION] [--team TEAM] [--assignee self|--unassign] [--state STATE] [--project PROJECT|--remove-project] [--milestone MILESTONE|--remove-milestone] [--priority 0..4]
workrus issue delete [ID|NUMBER] --confirm CANONICAL_ID [--dry-run]
workrus issue comment list [ID|NUMBER] [--limit N] [--after CURSOR]
workrus issue comment add [ID|NUMBER] --body TEXT [-p COMMENT_ID]
workrus issue comment update COMMENT_ID --body TEXT
workrus issue comment delete COMMENT_ID --confirm COMMENT_ID [--dry-run]
workrus issue start [ID|NUMBER]
workrus issue id|title|url [ID|NUMBER]
```

`--json` is accepted anywhere in executable commands and writes one JSON document to stdout. `--help` and `--version` stay plain text even with `--json`; completions are also plain text and reject `--json`. Successful commands exit 0; input, authentication, and configuration errors exit 2; operational failures exit 1. Errors go to stderr, except documented JSON partial results.

Source the deterministic generated script for your shell, for example `source <(workrus completion bash)`. Scripts complete the command catalogue and its resource subcommands without invoking Linear. The Bash script uses only Bash built-ins (not the optional `bash-completion` `_init_completion` helper).

## Behavior and safeguards

`--limit 0` fetches cursor pages until exhaustion; positive limits cap aggregate results. Browser URLs must be HTTPS. `--web` uses the platform launcher; `--app` targets the Linear desktop app only on macOS and is rejected elsewhere. `issue pr` uses `gh pr create` without a shell and prints only its validated HTTPS result URL. Git, `gh`, and launchers have `LINEAR_API_KEY` and Git-routing variables removed; Git branch switching disables repository hooks.

`issue start` is Git-first: it creates or switches to Linear's branch before moving the issue to its unique `started` state. It re-reads the issue after the Git change and reports a retryable partial result if Linear work then fails. It never resets, deletes, pushes, or fetches.

Create prompts run only with a human TTY and never contaminate JSON stdout. Destructive issue, comment, milestone, and document operations require exact `--confirm`; `--dry-run` validates and resolves without mutation. Issue deletion archives, and document deletion trashes, rather than permanently deleting. Project and milestone names resolve exactly and ambiguity reports candidate IDs. Project `--priority` and `--label` are rejected because their public create-input fields are not confirmed.

Documents support only public issue and project targets. Content is bounded UTF-8 Markdown from a regular `--content-file` or `--stdin`, never ProseMirror/Yjs fields. Content/target replacement checks inline-comment metadata and requires `--force` when active anchors exist. `--permanent` and team/cycle/initiative/release document targets are rejected. `team autolinks` is a feature-unavailable compatibility stub.

## Deliberate exclusions

Git is fixed: there is no `jj`, VCS configuration, commit discovery, keyring, pager, or alternate authentication. Image download, image preview, public image upload, public asset URLs, and image-specific behavior are unsupported. Generic private non-image attachments are **deferred** until their complete private-upload lifecycle can be verified against supported official Linear APIs; workrus does not expose attachment commands or use internal Linear APIs.

## License

MIT, Copyright Alvin Yip. See [LICENSE](LICENSE).

Global configuration writes use same-directory atomic replacement on Unix and Windows. Configuration and document content reads reject final-component symlinks or Windows reparse points and enforce their size bounds on the opened handle.
