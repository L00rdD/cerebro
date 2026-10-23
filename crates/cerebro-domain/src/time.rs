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

/// Formats an elapsed time for the session cards: `"42s"`, `"9min"`, `"2h05"`, `"3d"`.
pub fn humanize_duration(elapsed_ms: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;

    let seconds = elapsed_ms / 1_000;
    match seconds {
        0..MINUTE => format!("{seconds}s"),
        MINUTE..HOUR => format!("{}min", seconds / MINUTE),
        HOUR..DAY => format!("{}h{:02}", seconds / HOUR, (seconds % HOUR) / MINUTE),
        _ => format!("{}d", seconds / DAY),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn since_saturates() {
        // Act & Assert
        assert_eq!(Timestamp(10).since(Timestamp(4)), 6);
        assert_eq!(Timestamp(4).since(Timestamp(10)), 0);
    }

    #[test]
    fn humanize_seconds() {
        // Act & Assert
        assert_eq!(humanize_duration(0), "0s");
        assert_eq!(humanize_duration(42_999), "42s");
    }

    #[test]
    fn humanize_minutes() {
        // Act & Assert
        assert_eq!(humanize_duration(60_000), "1min");
        assert_eq!(humanize_duration(59 * 60_000 + 59_000), "59min");
    }

    #[test]
    fn humanize_hours_pad_minutes() {
        // Act & Assert
        assert_eq!(humanize_duration(3_600_000), "1h00");
        assert_eq!(humanize_duration(2 * 3_600_000 + 5 * 60_000), "2h05");
    }

    #[test]
    fn humanize_days() {
        // Act & Assert
        assert_eq!(humanize_duration(3 * 24 * 3_600_000 + 1), "3d");
    }
}
