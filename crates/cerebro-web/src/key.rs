use std::fmt::{self, Write as _};
use std::fs;
use std::io;
use std::path::Path;

/// Secret part of the phone link (`?key=…`). Anyone on the private network can reach the
/// server, only holders of the link can read the sessions.
#[derive(Clone, PartialEq, Eq)]
pub struct AccessKey(String);

impl AccessKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Reads the key saved at `path`, or creates one the first time. A saved key keeps
    /// bookmarks working across restarts; delete the file to revoke every link.
    pub fn load_or_create(path: &Path) -> io::Result<Self> {
        if let Ok(saved) = fs::read_to_string(path) {
            let saved = saved.trim();
            if !saved.is_empty() {
                return Ok(Self::new(saved));
            }
        }
        let key = Self::random()?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, &key.0)?;
        restrict_to_owner(path)?;
        Ok(key)
    }

    fn random() -> io::Result<Self> {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(io::Error::other)?;
        let hex = bytes
            .iter()
            .fold(String::with_capacity(32), |mut hex, byte| {
                let _ = write!(hex, "{byte:02x}");
                hex
            });
        Ok(Self(hex))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Compares in constant time, so response delays don't reveal the key byte by byte.
    pub fn matches(&self, candidate: &str) -> bool {
        let expected = self.0.as_bytes();
        let candidate = candidate.as_bytes();
        expected.len() == candidate.len()
            && expected
                .iter()
                .zip(candidate)
                .fold(0, |difference, (left, right)| difference | (left ^ right))
                == 0
    }
}

/// Never print the key itself in logs or debug output.
impl fmt::Debug for AccessKey {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str("AccessKey(…)")
    }
}

#[cfg(unix)]
fn restrict_to_owner(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
#[expect(
    clippy::unnecessary_wraps,
    reason = "same signature as the unix version"
)]
fn restrict_to_owner(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_key_once_then_reuses_it() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cerebro/mobile-key");

        // Act
        let first = AccessKey::load_or_create(&path).unwrap();
        let second = AccessKey::load_or_create(&path).unwrap();

        // Assert
        assert_eq!(first.as_str().len(), 32);
        assert_eq!(first, second);
    }

    #[test]
    fn matches_only_the_exact_key() {
        // Arrange
        let key = AccessKey::new("abc123");

        // Act & Assert
        assert!(key.matches("abc123"));
        assert!(!key.matches("abc124"));
        assert!(!key.matches("abc"));
        assert!(!key.matches(""));
    }
}
