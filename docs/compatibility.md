# Compatibility

workrus retains familiar issue-oriented names from `schpet/linear-cli`: configuration, team list/id/members/create, user list, issue list/query/view/create/update/start, branch-inferred issue scalars, and `issue pr`/`pull-request`. It is an independent implementation, not a drop-in replacement.

Supported expansion also includes projects, milestones (`milestone` and `m`), public issue/project documents (`document` and `docs`), issue comments, browser/app opening, and deterministic `completion` (with `completions` retained as an alias) scripts for bash, zsh, fish, and PowerShell. `team autolinks` remains a compatibility stub that returns feature-unavailable because Linear has no public autolink management API.

Deliberate differences: configuration is global XDG/AppData TOML only; Git is fixed and repository hooks are disabled for branch switching; `LINEAR_API_KEY` is environment-only. There is no `jj`, VCS configuration, commit discovery, keyring, pager, interactive authentication, or broad filtering. Completion output is plain text, has no runtime Linear dependency, and rejects `--json`.

Documents exclude internal team/cycle/initiative/release targets, permanent deletion, and inline rich-text anchor creation. Images and image-specific behavior are unsupported. Generic private non-image attachments are explicitly **deferred**, rather than implemented through a path-racy uploader or an internal Linear API. Labels and project create priority are not advertised as supported because their public mutation inputs are unconfirmed.
