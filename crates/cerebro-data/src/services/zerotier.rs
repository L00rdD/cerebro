use std::io;
use std::path::Path;
use std::sync::Arc;

use cerebro_domain::DomainError;
use cerebro_domain::ports::{NetworkStatus, PrivateNetwork};

use super::{CommandOutput, CommandRunner};
use crate::models::{ZeroTierInfo, ZeroTierNetworkEntry};

/// Where installers put `zerotier-cli`. An app started from the Finder doesn't get the shell's
/// `PATH`, so known locations are tried before the bare name.
const CLI_CANDIDATES: &[&str] = if cfg!(target_os = "macos") {
    &[
        "/usr/local/bin/zerotier-cli",
        "/opt/homebrew/bin/zerotier-cli",
    ]
} else if cfg!(windows) {
    &[r"C:\ProgramData\ZeroTier\One\zerotier-cli.bat"]
} else {
    &["/usr/sbin/zerotier-cli", "/usr/bin/zerotier-cli"]
};

/// `ZeroTier`, queried through `zerotier-cli`.
pub struct ZeroTierNetwork {
    runner: Arc<dyn CommandRunner>,
    cli: String,
}

impl ZeroTierNetwork {
    pub fn new(runner: Arc<dyn CommandRunner>) -> Self {
        let cli = CLI_CANDIDATES
            .iter()
            .find(|candidate| Path::new(candidate).exists())
            .map_or("zerotier-cli", |candidate| candidate);
        Self::with_cli(runner, cli)
    }

    pub fn with_cli(runner: Arc<dyn CommandRunner>, cli: impl Into<String>) -> Self {
        Self {
            runner,
            cli: cli.into(),
        }
    }

    fn query(&self, command: &str) -> Result<CommandOutput, DomainError> {
        self.runner
            .run(&self.cli, &["-j", command])
            .map_err(|error| DomainError::Platform(format!("{}: {error}", self.cli)))
    }
}

impl PrivateNetwork for ZeroTierNetwork {
    fn id(&self) -> &'static str {
        "zerotier"
    }

    fn name(&self) -> String {
        "ZeroTier".into()
    }

    /// Encrypted end to end and reachable from anywhere: the network Cerebro was built for.
    fn on_by_default(&self) -> bool {
        true
    }

    fn status(&self) -> Result<NetworkStatus, DomainError> {
        let info = match self.runner.run(&self.cli, &["-j", "info"]) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(NetworkStatus::NotInstalled);
            }
            Err(error) => return Err(DomainError::Platform(format!("{}: {error}", self.cli))),
            Ok(output) if !output.success => return stopped_or_error(&output),
            Ok(output) => parse::<ZeroTierInfo>(&output.stdout)?,
        };
        if !info.online {
            return Ok(NetworkStatus::Offline);
        }

        let output = self.query("listnetworks")?;
        if !output.success {
            return stopped_or_error(&output);
        }
        let networks = parse::<Vec<ZeroTierNetworkEntry>>(&output.stdout)?;
        Ok(NetworkStatus::Online(
            networks
                .iter()
                .flat_map(ZeroTierNetworkEntry::private_addresses)
                .collect(),
        ))
    }

    fn start(&self) -> Result<(), DomainError> {
        let (program, args): (&str, &[&str]) = if cfg!(target_os = "macos") {
            // The service is a launchd daemon: starting it needs the administrator password.
            (
                "osascript",
                &[
                    "-e",
                    "do shell script \"launchctl load -w /Library/LaunchDaemons/com.zerotier.one.plist; \
                     launchctl kickstart system/com.zerotier.one\" with administrator privileges",
                ],
            )
        } else if cfg!(windows) {
            return Err(DomainError::Platform(
                "start the \"ZeroTier One\" service from the Services app".into(),
            ));
        } else {
            ("pkexec", &["systemctl", "start", "zerotier-one"])
        };
        let output = self
            .runner
            .run(program, args)
            .map_err(|error| DomainError::Platform(format!("{program}: {error}")))?;
        if output.success {
            Ok(())
        } else {
            Err(DomainError::Platform(format!(
                "could not start ZeroTier: {}",
                output.stderr.trim()
            )))
        }
    }
}

/// `zerotier-cli` fails the same way whether the service is down or the user may not query it:
/// only the message tells them apart.
fn stopped_or_error(output: &CommandOutput) -> Result<NetworkStatus, DomainError> {
    let message = format!("{}{}", output.stdout, output.stderr);
    if message.contains("connection failed") || message.contains("missing port") {
        Ok(NetworkStatus::Stopped)
    } else {
        Err(DomainError::Platform(format!(
            "zerotier-cli: {}",
            message.trim()
        )))
    }
}

fn parse<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, DomainError> {
    serde_json::from_str(json)
        .map_err(|error| DomainError::Platform(format!("unexpected zerotier-cli output: {error}")))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;

    /// Answers by command line; anything else is "command not found".
    struct ScriptedRunner {
        answers: HashMap<String, CommandOutput>,
        calls: Mutex<Vec<String>>,
    }

    impl ScriptedRunner {
        fn new(answers: &[(&str, bool, &str)]) -> Arc<Self> {
            Arc::new(Self {
                answers: answers
                    .iter()
                    .map(|(command, success, text)| {
                        (
                            (*command).to_owned(),
                            CommandOutput {
                                success: *success,
                                stdout: if *success {
                                    (*text).into()
                                } else {
                                    String::new()
                                },
                                stderr: if *success {
                                    String::new()
                                } else {
                                    (*text).into()
                                },
                            },
                        )
                    })
                    .collect(),
                calls: Mutex::new(Vec::new()),
            })
        }
    }

    impl CommandRunner for ScriptedRunner {
        fn run(&self, program: &str, args: &[&str]) -> io::Result<CommandOutput> {
            let key = std::iter::once(program)
                .chain(args.iter().copied())
                .collect::<Vec<_>>()
                .join(" ");
            self.calls.lock().unwrap().push(key.clone());
            self.answers
                .get(&key)
                .cloned()
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        }
    }

    const NETWORKS: &str = r#"[
        {"status": "OK", "type": "PRIVATE", "assignedAddresses": ["10.147.17.42/16"], "name": "home"},
        {"status": "OK", "type": "PUBLIC", "assignedAddresses": ["192.168.195.10/24"], "name": "open"}
    ]"#;

    fn status(runner: Arc<ScriptedRunner>) -> Result<NetworkStatus, DomainError> {
        ZeroTierNetwork::with_cli(runner, "zt").status()
    }

    #[test]
    fn online_with_the_private_addresses_only() {
        // Arrange
        let runner = ScriptedRunner::new(&[
            (
                "zt -j info",
                true,
                r#"{"online": true, "address": "0123456789"}"#,
            ),
            ("zt -j listnetworks", true, NETWORKS),
        ]);

        // Act
        let result = status(runner);

        // Assert
        assert_eq!(
            result,
            Ok(NetworkStatus::Online(vec!["10.147.17.42".parse().unwrap()]))
        );
    }

    #[test]
    fn offline_while_the_node_is_not_connected() {
        // Arrange
        let runner = ScriptedRunner::new(&[("zt -j info", true, r#"{"online": false}"#)]);

        // Act & Assert
        assert_eq!(status(runner), Ok(NetworkStatus::Offline));
    }

    #[test]
    fn stopped_when_the_service_does_not_answer() {
        // Arrange
        let runner = ScriptedRunner::new(&[(
            "zt -j info",
            false,
            "Error connecting to the ZeroTier service: connection failed",
        )]);

        // Act & Assert
        assert_eq!(status(runner), Ok(NetworkStatus::Stopped));
    }

    #[test]
    fn not_installed_without_the_cli() {
        // Act & Assert
        assert_eq!(
            status(ScriptedRunner::new(&[])),
            Ok(NetworkStatus::NotInstalled)
        );
    }

    #[test]
    fn a_permission_problem_is_an_error_not_a_stopped_service() {
        // Arrange
        let runner = ScriptedRunner::new(&[(
            "zt -j info",
            false,
            "authtoken.secret not found or readable in /var/lib/zerotier-one",
        )]);

        // Act
        let result = status(runner);

        // Assert
        assert!(
            matches!(result, Err(DomainError::Platform(message)) if message.contains("authtoken"))
        );
    }
}
