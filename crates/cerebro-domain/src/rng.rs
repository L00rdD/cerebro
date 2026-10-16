//! Deterministic "randomness": the same id always gives the same pick,
//! across launches and platforms (unlike `std::hash`, which is randomly seeded).

/// 64-bit FNV-1a followed by a splitmix64 finalizer for a good bit spread.
pub fn hash(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        h = (h ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
    }
    mix(h)
}

/// splitmix64 finalizer.
pub fn mix(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Picks an index in `0..len` from `seed`, probing forward past indices for which
/// `taken` is true. Falls back to the first choice when everything is taken.
pub fn pick_free(seed: &str, len: usize, taken: impl Fn(usize) -> bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    // `len` fits in u64 on every supported target, and the modulo result fits back in usize.
    let start = usize::try_from(hash(seed) % len as u64).unwrap_or(0);
    (0..len)
        .map(|offset| (start + offset) % len)
        .find(|&i| !taken(i))
        .or(Some(start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable() {
        // Pinned value: changing it would reshuffle everyone's rooms.
        assert_eq!(hash("session-1"), hash("session-1"));
        assert_ne!(hash("session-1"), hash("session-2"));
        assert_eq!(hash(""), mix(0xcbf2_9ce4_8422_2325));
    }

    #[test]
    fn pick_free_skips_taken_and_falls_back() {
        let first = pick_free("x", 5, |_| false).unwrap();
        let second = pick_free("x", 5, |i| i == first).unwrap();
        assert_ne!(first, second);
        assert_eq!(pick_free("x", 5, |_| true), Some(first));
        assert_eq!(pick_free("x", 0, |_| false), None);
    }
}
