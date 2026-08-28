# Publishing

`.github/workflows/release.yml` runs Release-plz on every push to `master`.

- `release-plz release` publishes an unpublished Cargo version, creates its tag, and creates the GitHub release.
- `release-plz release-pr` maintains the version/changelog release PR.
- The publishing job uses crates.io Trusted Publishing through GitHub OIDC; no long-lived Cargo token is stored in GitHub.

## One-time setup

1. Create a protected GitHub Actions environment named `cargo-release`.
2. Publish `workrus` once with a narrowly scoped crates.io token. crates.io does not permit Trusted Publishing to bootstrap a crate name that does not exist yet.
3. In the `workrus` settings on crates.io, add a GitHub Trusted Publisher with:

   | Field | Value |
   |---|---|
   | Repository owner | `keongalvin` |
   | Repository name | `workrus-cli` |
   | Workflow filename | `release.yml` |
   | Environment | `cargo-release` |

4. Enable Trusted Publishing enforcement after verifying the first automated release.

The crate name `workrus` was unclaimed when this workflow was configured. The first manual publish is the only bootstrap step; subsequent versions are published by Release-plz using short-lived OIDC credentials.

## Release process

1. Merge normal changes into `master`; CI must pass.
2. Review and merge the PR maintained by `release-plz release-pr`.
3. On the resulting `master` push, `release-plz release` runs the release checks, publishes the crate, creates the tag, and creates the GitHub release.

Published crate versions are immutable. Do not reuse or move a release tag after publication.
