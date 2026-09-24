---
name: release
description: Release a new version of Scraps from start to finish. Opens and merges the version-bump PR, publishes the GitHub Release with categorized notes, then follows the workflow that ships the binaries, crates.io, Homebrew and the floating tags. Use this whenever the user asks to release, cut, ship or publish a Scraps version (e.g. "release 3.1.0", "v3.0.1 をリリースして"), or to finish a release that stopped halfway, such as a merged release PR with no GitHub Release yet.
---

# Release

Takes a version such as `3.1.0` (semver, no `v`); the tag is `v3.1.0`. If the user gave none, propose one from what is shipping (step 1): a breaking change means major, a feature means minor, anything else patch. Confirm it before going on.

Two constraints shape the flow:

- `main` takes changes only through a PR that passes the `build` and `zizmor` checks and has one approval, so the version bump travels as a release PR.
- Publishing the GitHub Release runs `.github/workflows/release.yml`. It uploads the binaries, publishes both crates to crates.io, updates `boykush/homebrew-tap` and moves the `v{major}` / `v{major}.{minor}` tags. None of that can be taken back.

Get the user's go-ahead before merging and before publishing. In between, keep going without stopping. Draft the release notes yourself: the user reviews them as a whole before publishing.

A release can stop halfway, for example when a PR is waiting on checks or a session ends. Check what already exists before starting, and resume from the first step that hasn't happened:

```bash
git fetch origin main --tags
gh pr list --head release/v<version> --state all --json number,state,mergeCommit
gh release view v<version>
```

## 1. See what is shipping

```bash
prev=$(git tag --list 'v[0-9]*.[0-9]*.[0-9]*' --merged origin/main --sort=-v:refname | grep -v -- - | grep -vx 'v<version>' | head -1)
gh api repos/boykush/scraps/releases/generate-notes -f tag_name=v<version> -f target_commitish=main -f previous_tag_name="$prev" --jq .body
```

The tag pattern skips the floating `v3` / `v3.0` tags, and `grep -v -- -` skips pre-releases. `generate-notes` returns every PR merged since `$prev`, whether it was merged or squashed (Renovate's are squashed), and only formats text: it creates nothing. Read the PRs that matter with `gh pr view <N> --json title,body`. A `!` in the title or `BREAKING CHANGE` in the body marks a breaking change.

## 2. Open the release PR

Branch from `origin/main` in the current checkout:

```bash
git status --porcelain                 # must print nothing
git switch -c release/v<version> origin/main
```

If the checkout has uncommitted changes, stop and ask. Don't `git stash`: every worktree of this repository shares the stash stack. Don't `git checkout main` either: it fails while another worktree has `main` checked out.

Bump the version in three places, then build so that `Cargo.lock` picks up both `scraps` and `scraps_libs`:

- `Cargo.toml`: `version` under `[workspace.package]` and under `[workspace.dependencies.scraps_libs]`
- `modules/libs/Cargo.toml`: `version`
- `mise run cargo:build`

The diff should touch those two manifests and `Cargo.lock`, nothing else. Commit it with the message `v<version>` and push it with `git push -u origin release/v<version>`. Then open the PR with `gh pr create --base main --title "v<version>"`. Start the body with "Version bump for v<version>." and summarize what ships from step 1. Put breaking changes first, saying how to keep the old behaviour and why it changed. The release notes grow out of this summary.

## 3. Merge it

1. `gh pr checks <number> --watch --required` waits for `build` and `zizmor`.
2. The approval comes from the `approve-pr` workflow, which approves PRs by `boykush` as soon as they open. Check `gh pr view <number> --json reviewDecision`. If it stays `REVIEW_REQUIRED`, look at that workflow's run and tell the user. Don't look for another way to approve.
3. With both in place, ask the user whether to merge.
4. On a yes, run `gh pr merge <number> --merge`. A merge commit keeps the release as one commit on `main`, and the tag goes on it.

## 4. Publish

The release commit is the PR's merge commit, not the tip of `main`, which may have moved on:

```bash
sha=$(gh pr view <number> --json mergeCommit --jq .mergeCommit.oid)
git fetch origin main --tags
git show "$sha:Cargo.toml" | grep -m1 '^version'   # must be <version>
```

Run `generate-notes` again with `-f target_commitish="$sha"`, since PRs merged while the release PR was open ship too. Write the notes in the shape of earlier releases; `gh release view v3.0.0` is a good one to match.

- If there is a breaking change, the notes open with `## ⚠️ Breaking change`: what changed, how to keep the old behaviour, and why. Leave the section out otherwise.
- `## What's Changed` has one line per PR, `<PR title> (#N)`, filed by the title's prefix. Show only the sections that have entries.

  | Prefix | Section |
  | --- | --- |
  | `feat` | ✨ Features |
  | `fix` | 🐛 Bug Fixes |
  | `perf` | ⚡ Performance |
  | `docs` | 📚 Documentation |
  | `refactor` | 🔧 Refactoring |
  | `test` | ✅ Tests |
  | anything else, and the release PR itself | 🔧 Maintenance |

- Under a feature, a fix users would notice, or a performance change, add a short paragraph from the PR body. Say what a user will notice, with numbers for performance. Routine maintenance stays on one line, and dependency updates can share one.
- End with `**Full Changelog**: https://github.com/boykush/scraps/compare/<prev>...v<version>`.

Write the notes to a file outside the repository, because `--notes-file` keeps backticks and `$` away from the shell. Show the user the notes, the commit and what publishing sets off. On a yes:

```bash
gh release create v<version> --target "$sha" --title "v<version>" --notes-file <file>
```

This creates the tag on GitHub at `$sha`, so nothing is tagged or pushed locally. A pre-release version (`-rc.N`) also takes `--prerelease`. The workflow then only builds binaries.

## 5. Follow the release workflow

```bash
gh run list --workflow=release.yml --limit 3 --json databaseId,headBranch,status
gh run watch <run-id> --exit-status
```

Use the run whose `headBranch` is `v<version>`. `Build` uploads the five binaries. `Update homebrew formula`, `Update v{major} and v{major}.{minor} tags` and `Publish to crates.io` each start after `Build`. Any of them can fail alone and leave the release half-published, so report every job.

When one fails, show `gh run view <run-id> --log-failed` and let the user decide. Re-running is not always safe. `cargo publish` rejects a version that is already on crates.io, so a re-run fails after `scraps_libs` has gone out.

## 6. Hand off

Give the user the release URL. Then suggest the `doc-update` skill to check the docs against what shipped.
