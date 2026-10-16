//! Time helpers.

/// Milliseconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn millis(self) -> u64 {
        self.0
    }

    /// Time elapsed since `earlier`, saturating at zero when clocks disagree.
    pub fn since(self, earlier: Timestamp) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

/// EXERCISE 1 (see `EXERCISES.md`): format an elapsed time for the session cards.
///
/// Expected output: `"42s"`, `"9min"`, `"2h05"`, `"3d"`.
pub fn humanize_duration(elapsed_ms: u64) -> String {
    todo!("Exercise 1: format {elapsed_ms} ms as a short human duration")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn since_saturates() {
        assert_eq!(Timestamp(10).since(Timestamp(4)), 6);
        assert_eq!(Timestamp(4).since(Timestamp(10)), 0);
    }

    #[test]
    #[ignore = "exercise 1"]
    fn humanize_seconds() {
        assert_eq!(humanize_duration(0), "0s");
        assert_eq!(humanize_duration(42_999), "42s");
    }

    #[test]
    #[ignore = "exercise 1"]
    fn humanize_minutes() {
        assert_eq!(humanize_duration(60_000), "1min");
        assert_eq!(humanize_duration(59 * 60_000 + 59_000), "59min");
    }

    #[test]
    #[ignore = "exercise 1"]
    fn humanize_hours_pad_minutes() {
        assert_eq!(humanize_duration(3_600_000), "1h00");
        assert_eq!(humanize_duration(2 * 3_600_000 + 5 * 60_000), "2h05");
    }

    #[test]
    #[ignore = "exercise 1"]
    fn humanize_days() {
        assert_eq!(humanize_duration(3 * 24 * 3_600_000 + 1), "3d");
    }
}
