use std::fmt;

/// Declares a string newtype so a `RoomId` can never be passed where a `SessionId` is expected.
macro_rules! id_type {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
                output.write_str(&self.0)
            }
        }
    };
}

id_type!(
    /// AI coding tool a session runs in, e.g. `claude-code`. Chosen by the data source.
    ToolId
);
id_type!(
    /// Session id, unique within its tool (a UUID for Claude Code).
    SessionId
);
id_type!(
    /// Subagent id, unique within its tool.
    AgentId
);
id_type!(
    /// Key of a character sprite in the asset catalog.
    CharacterId
);
id_type!(
    /// Key of a room in the asset catalog.
    RoomId
);
