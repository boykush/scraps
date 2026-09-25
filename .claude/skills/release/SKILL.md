---
name: release
description: Release a new version of Scraps from start to finish. Opens and merges the version-bump PR, publishes the GitHub Release with categorized notes, follows the workflow that ships the binaries, crates.io and Homebrew, then moves the floating v{major} / v{major}.{minor} tags. Use this whenever the user asks to release, cut, ship or publish a Scraps version (e.g. "release 3.1.0", "v3.0.1 をリリースして"), or to finish a release that stopped halfway, such as a merged release PR with no GitHub Release yet or floating tags still on the previous version.
---

# Release

Takes a version such as `3.1.0` (semver, no `v`); the tag is `v3.1.0`. If the user gave none, propose one from what is shipping (step 1): a breaking change means major, a feature means minor, anything else patch. Confirm it before going on.

Three constraints shape the flow:

- `main` takes changes only through a PR that passes the `build` and `zizmor` checks and has one approval, so the version bump travels as a release PR.
- Publishing the GitHub Release runs `.github/workflows/release.yml`. It uploads the binaries, publishes both crates to crates.io and updates `boykush/homebrew-tap`. None of that can be taken back.
- The floating `v{major}` / `v{major}.{minor}` tags are moved from this checkout with the user's credentials. `GITHUB_TOKEN` can't move them: GitHub refuses a tag whose commit has different `.github/workflows/` from the default branch, and any workflow change merged after the release commit makes them differ.

Get the user's go-ahead before merging, before publishing and before moving the floating tags. In between, keep going without stopping. Draft the release notes yourself: the user reviews them as a whole before publishing.

A release can stop halfway, for example when a PR is waiting on checks or a session ends. Check what already exists before starting, and resume from the first step that hasn't happened:

```bash
git fetch origin main --tags --force
gh pr list --head release/v<version> --state all --json number,state,mergeCommit
gh release view v<version>
git ls-remote origin refs/tags/v<version> refs/tags/v<major> refs/tags/v<major>.<minor>
```

Without `--force`, git refuses to update a tag it already has, so the fetch fails once the floating tags have moved. The floating tags are done when they point where `v<version>` does.

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
2. The approval comes from ai-review. Its `ai-review / review` check reviews the PR against the rules in boykush/adr and submits the verdict as a `claude[bot]` review: an approval when nothing is violated, a request for changes otherwise. The check isn't required, so the first command doesn't wait for it:

   ```bash
   gh run list --workflow=ai-review.yml --branch release/v<version> --limit 1 --json databaseId,status
   gh run watch <run-id> --exit-status
   gh pr view <number> --json reviewDecision
   ```

   If the run fails, or the decision is `CHANGES_REQUESTED` or `REVIEW_REQUIRED`, look at the run and the review, and tell the user. Don't look for another way to approve. The owner can bypass the approval, but that is for the user to do: never run `gh pr merge --admin`.
3. With both in place, ask the user whether to merge.
4. On a yes, run `gh pr merge <number> --merge`. A merge commit keeps the release as one commit on `main`, and the tag goes on it.

## 4. Publish

The release commit is the PR's merge commit, not the tip of `main`, which may have moved on:

```bash
sha=$(gh pr view <number> --json mergeCommit --jq .mergeCommit.oid)
git fetch origin main --tags --force
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

Use the run whose `headBranch` is `v<version>`. `Build` uploads the five binaries. `Update homebrew formula` and `Publish to crates.io` each start after `Build`. Any of them can fail alone and leave the release half-published, so report every job.

When one fails, show `gh run view <run-id> --log-failed` and let the user decide. Re-running is not always safe. `cargo publish` rejects a version that is already on crates.io, so a re-run fails after `scraps_libs` has gone out.

## 6. Move the floating tags

Skip this for a pre-release. Users of the setup action pin `boykush/scraps@v<major>` or `@v<major>.<minor>`, and the action installs the binaries of the version its ref carries. Move the tags once `Build` has succeeded, whatever happened to the other jobs. Check where they point now:

```bash
git ls-remote origin refs/tags/v<major> refs/tags/v<major>.<minor>
```

Show the user both tags' current commits and `$sha` from step 4. On a yes, move both in one atomic push. Each lease is the commit its tag points at now, or empty for a tag that doesn't exist yet, so nothing moves if either tag changed after you looked:

```bash
git push --atomic \
  --force-with-lease=refs/tags/v<major>:<current commit> \
  --force-with-lease=refs/tags/v<major>.<minor>:<current commit, or empty> \
  origin "${sha}:refs/tags/v<major>" "${sha}:refs/tags/v<major>.<minor>"
git fetch origin --tags --force
```

Keep the braces in `${sha}`: zsh reads the `:r` after a bare `$sha` as a modifier and mangles the refspec.

## 7. Hand off

Give the user the release URL. Then suggest the `doc-update` skill to check the docs against what shipped.
