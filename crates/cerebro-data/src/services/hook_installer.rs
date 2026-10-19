use std::fs;
use std::path::PathBuf;

use cerebro_domain::DomainError;
use cerebro_domain::ports::{HookInstaller, HookSetup};
use cerebro_protocol::{HOOK_EVENTS, HOOK_MARKER, now_ms};
use serde_json::{Map, Value, json};

/// Manages Cerebro's entries in Claude Code's `settings.json`, leaving everything else untouched.
pub struct ClaudeSettingsHookInstaller {
    settings_file: PathBuf,
    hook_binary: PathBuf,
}

impl ClaudeSettingsHookInstaller {
    pub fn new(settings_file: impl Into<PathBuf>, hook_binary: impl Into<PathBuf>) -> Self {
        Self {
            settings_file: settings_file.into(),
            hook_binary: hook_binary.into(),
        }
    }

    /// The exact command string written to the settings (quoted when the path has spaces).
    fn command(&self) -> String {
        let path = self.hook_binary.display().to_string();
        if path.contains(' ') {
            format!("\"{path}\"")
        } else {
            path
        }
    }

    fn read_settings(&self) -> Result<Value, DomainError> {
        match fs::read_to_string(&self.settings_file) {
            Ok(text) => serde_json::from_str(&text).map_err(|error| {
                DomainError::Platform(format!(
                    "{} is not valid JSON: {error}",
                    self.settings_file.display()
                ))
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
            Err(error) => Err(DomainError::Platform(error.to_string())),
        }
    }
}

impl HookInstaller for ClaudeSettingsHookInstaller {
    fn status(&self) -> Result<HookSetup, DomainError> {
        let settings = self.read_settings()?;
        let commands = cerebro_commands(&settings);
        if commands.is_empty() {
            return Ok(HookSetup::Missing);
        }
        let expected = self.command();
        let complete = HOOK_EVENTS.iter().all(|event| {
            commands
                .iter()
                .filter(|(hook_event, _)| hook_event == event)
                .map(|(_, command)| command)
                .eq([&expected])
        });
        let no_strays = commands.iter().all(|(hook_event, command)| {
            *command == expected && HOOK_EVENTS.contains(&hook_event.as_str())
        });
        Ok(if complete && no_strays {
            HookSetup::Installed
        } else {
            HookSetup::Outdated
        })
    }

    fn install(&self) -> Result<(), DomainError> {
        let platform = |error: std::io::Error| DomainError::Platform(error.to_string());
        let mut settings = self.read_settings()?;
        if self.settings_file.exists() {
            let backup = self
                .settings_file
                .with_extension(format!("json.bak-cerebro-{}", now_ms()));
            fs::copy(&self.settings_file, backup).map_err(platform)?;
        }

        let root = settings
            .as_object_mut()
            .ok_or_else(|| DomainError::Platform("settings.json must contain an object".into()))?;
        let hooks = root.entry("hooks").or_insert_with(|| json!({}));
        let hooks = hooks
            .as_object_mut()
            .ok_or_else(|| DomainError::Platform("\"hooks\" must be an object".into()))?;

        remove_cerebro_entries(hooks);
        for event in HOOK_EVENTS {
            let groups = hooks.entry(*event).or_insert_with(|| json!([]));
            if let Some(groups) = groups.as_array_mut() {
                groups.push(json!({ "hooks": [{ "type": "command", "command": self.command() }] }));
            }
        }

        if let Some(parent) = self.settings_file.parent() {
            fs::create_dir_all(parent).map_err(platform)?;
        }
        let text = serde_json::to_string_pretty(&settings)
            .map_err(|error| DomainError::Platform(error.to_string()))?;
        fs::write(&self.settings_file, text + "\n").map_err(platform)
    }
}

/// `(event, command)` for every hook command that belongs to Cerebro.
fn cerebro_commands(settings: &Value) -> Vec<(String, String)> {
    let Some(hooks) = settings.get("hooks").and_then(Value::as_object) else {
        return Vec::new();
    };
    hooks
        .iter()
        .flat_map(|(event, groups)| {
            groups
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|group| group.get("hooks").and_then(Value::as_array))
                .flatten()
                .filter_map(|hook| hook.get("command").and_then(Value::as_str))
                .filter(|command| command.contains(HOOK_MARKER))
                .map(move |command| (event.clone(), command.to_owned()))
        })
        .collect()
}

/// Drops Cerebro hooks (any version), then groups and events left empty.
fn remove_cerebro_entries(hooks: &mut Map<String, Value>) {
    for groups in hooks.values_mut() {
        let Some(groups) = groups.as_array_mut() else {
            continue;
        };
        for group in groups.iter_mut() {
            if let Some(list) = group.get_mut("hooks").and_then(Value::as_array_mut) {
                list.retain(|hook| {
                    !hook
                        .get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|command| command.contains(HOOK_MARKER))
                });
            }
        }
        groups.retain(|group| {
            group
                .get("hooks")
                .and_then(Value::as_array)
                .is_none_or(|hook_list| !hook_list.is_empty())
        });
    }
    hooks.retain(|_, groups| {
        groups
            .as_array()
            .is_none_or(|group_list| !group_list.is_empty())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installer(dir: &tempfile::TempDir) -> ClaudeSettingsHookInstaller {
        ClaudeSettingsHookInstaller::new(
            dir.path().join("settings.json"),
            "/Apps/Cerebro.app/Contents/MacOS/cerebro-hook",
        )
    }

    #[test]
    fn installs_into_empty_settings_and_is_idempotent() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        let installer = installer(&dir);

        // Act & Assert
        assert_eq!(installer.status().unwrap(), HookSetup::Missing);

        // Act
        installer.install().unwrap();

        // Assert
        assert_eq!(installer.status().unwrap(), HookSetup::Installed);

        // Act
        installer.install().unwrap();

        // Assert
        let settings: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn migrates_legacy_script_and_keeps_foreign_hooks() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        let legacy = json!({
            "model": "opus",
            "hooks": {
                "Stop": [{ "hooks": [{ "type": "command", "command": "/x/iterm2/cc-status" }] }],
                "SubagentStart": [{ "hooks": [{ "type": "command", "command": "/p/cerebro/hooks/cerebro-hook.sh" }] }]
            }
        });
        fs::write(dir.path().join("settings.json"), legacy.to_string()).unwrap();
        let installer = installer(&dir);

        // Act & Assert
        assert_eq!(installer.status().unwrap(), HookSetup::Outdated);

        // Act
        installer.install().unwrap();

        // Assert
        assert_eq!(installer.status().unwrap(), HookSetup::Installed);

        let text = fs::read_to_string(dir.path().join("settings.json")).unwrap();
        let settings: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(settings["model"], "opus");
        assert!(text.contains("cc-status"));
        assert!(!text.contains("cerebro-hook.sh"));
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 2);
        // A backup of the original file is kept next to it.
        let backups = fs::read_dir(dir.path()).unwrap().filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("bak-cerebro")
        });
        assert_eq!(backups.count(), 1);
    }

    #[test]
    fn moved_binary_is_reported_outdated() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        installer(&dir).install().unwrap();
        let moved = ClaudeSettingsHookInstaller::new(
            dir.path().join("settings.json"),
            "/elsewhere/cerebro-hook",
        );

        // Act
        let status = moved.status().unwrap();

        // Assert
        assert_eq!(status, HookSetup::Outdated);
    }

    #[test]
    fn paths_with_spaces_are_quoted() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        let installer = ClaudeSettingsHookInstaller::new(
            dir.path().join("settings.json"),
            "/My Apps/cerebro-hook",
        );

        // Act
        let command = installer.command();

        // Assert
        assert_eq!(command, "\"/My Apps/cerebro-hook\"");
    }
}
