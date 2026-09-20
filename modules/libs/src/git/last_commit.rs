use std::collections::HashMap;

/// Paths come out relative to the repository root, not to the directory run
/// in: combined diffs ignore `--relative`. `-z` leaves quotes and non-ASCII
/// unescaped.
pub(super) const LOG_ARGS: &[&str] = &[
    // User config must not reshape what `timestamps` parses.
    "-c",
    "log.follow=false",
    "-c",
    "log.showSignature=false",
    "-c",
    "diff.relative=false",
    "log",
    "--root",
    "--no-renames",
    "--no-color",
    "--name-only",
    "-z",
    "--format=@%ct",
    // A merge names a file only when it differs from every parent, which is
    // when `git log -1 <file>` stops at that merge.
    "-c",
    "--",
    ".",
];

/// Each path's time in the newest commit naming it, the commit that
/// `git log -1 <path>` reports. Every commit prints `@<unix time>`, then the
/// paths it touched after a newline; each field ends in NUL, and a commit
/// naming no path leaves an empty one.
pub(super) fn timestamps(output: &[u8]) -> HashMap<String, i64> {
    let mut timestamps = HashMap::new();
    let mut commit_ts = None;
    for field in output.split(|&byte| byte == 0) {
        let field = field.strip_prefix(b"\n").unwrap_or(field);
        if field.is_empty() {
            continue;
        }
        match header_ts(field) {
            Some(ts) => commit_ts = Some(ts),
            None => {
                if let Some(ts) = commit_ts {
                    timestamps
                        .entry(String::from_utf8_lossy(field).into_owned())
                        .or_insert(ts);
                }
            }
        }
    }
    timestamps
}

fn header_ts(field: &[u8]) -> Option<i64> {
    std::str::from_utf8(field.strip_prefix(b"@")?)
        .ok()?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected(entries: &[(&str, i64)]) -> HashMap<String, i64> {
        entries
            .iter()
            .map(|(path, ts)| (path.to_string(), *ts))
            .collect()
    }

    #[test]
    fn it_takes_each_path_from_the_newest_commit_naming_it() {
        let output = b"@300\0\na.md\0@200\0\na.md\0ctx/b.md\0";

        assert_eq!(
            timestamps(output),
            expected(&[("a.md", 300), ("ctx/b.md", 200)])
        );
    }

    #[test]
    fn it_skips_a_merge_that_names_no_path() {
        let output = b"@300\0\0@200\0\na.md\0";

        assert_eq!(timestamps(output), expected(&[("a.md", 200)]));
    }

    #[test]
    fn it_keeps_paths_verbatim() {
        let output = "@100\0\n\"Swarming\" 入門.md\0ctx/with space.md\0".as_bytes();

        assert_eq!(
            timestamps(output),
            expected(&[("\"Swarming\" 入門.md", 100), ("ctx/with space.md", 100)])
        );
    }

    #[test]
    fn it_reads_nothing_from_an_empty_log() {
        assert_eq!(timestamps(b""), HashMap::new());
    }
}
