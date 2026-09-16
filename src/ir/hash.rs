/// FNV-1a, 64-bit: stable across toolchains, dependency-free, and fast
/// enough for a few thousand small files. A collision would keep a stale
/// object for a changed source, at 2^-64 per change.
pub fn content_hash(bytes: &[u8]) -> String {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::empty("", "cbf29ce484222325")]
    #[case::a("a", "af63dc4c8601ec8c")]
    #[case::foobar("foobar", "85944171f73967e8")]
    fn it_matches_the_reference_vectors(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(content_hash(input.as_bytes()), expected);
    }

    #[test]
    fn it_separates_close_inputs() {
        assert_ne!(content_hash(b"# A\n"), content_hash(b"# B\n"));
    }
}
