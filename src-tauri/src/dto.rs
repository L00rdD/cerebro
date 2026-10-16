//! View models sent to the UI. The domain stays free of serde; mapping happens here.

use cerebro_domain::entities::{DataMode, Origin, Scene, Snapshot};
use cerebro_domain::ports::{FocusOutcome, HookSetup};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDto {
    pub scenes: Vec<SceneDto>,
    pub mode: ModeDto,
    pub counts: CountsDto,
    pub taken_at: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ModeDto {
    Registry,
    HooksFallback { reason: String },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountsDto {
    pub sessions: usize,
    pub busy: usize,
    pub agents: usize,
    pub characters: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneDto {
    pub id: String,
    pub name: String,
    pub cwd: String,
    pub status: String,
    pub status_since: u64,
    pub started_at: u64,
    /// False for hook-only sessions (no pid to follow).
    pub focusable: bool,
    pub room: String,
    pub actors: Vec<ActorDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDto {
    /// Stable id used to seed the actor's wandering path.
    pub id: String,
    pub character: String,
    /// `None` for the session lead, the subagent type otherwise.
    pub agent_kind: Option<String>,
}

impl From<&Snapshot> for SnapshotDto {
    fn from(snapshot: &Snapshot) -> Self {
        Self {
            scenes: snapshot.scenes.iter().map(SceneDto::from).collect(),
            mode: match &snapshot.mode {
                DataMode::Registry => ModeDto::Registry,
                DataMode::HooksFallback { reason } => ModeDto::HooksFallback { reason: reason.clone() },
            },
            counts: CountsDto {
                sessions: snapshot.scenes.len(),
                busy: snapshot.busy_count(),
                agents: snapshot.agent_count(),
                characters: snapshot.character_count(),
            },
            taken_at: snapshot.taken_at.millis(),
        }
    }
}

impl From<&Scene> for SceneDto {
    fn from(scene: &Scene) -> Self {
        let session = &scene.session;
        let lead = ActorDto {
            id: format!("lead:{}", session.id),
            character: scene.cast.lead.to_string(),
            agent_kind: None,
        };
        let agents = session.agents.iter().zip(&scene.cast.agents).map(|(agent, character)| ActorDto {
            id: format!("agent:{}", agent.id),
            character: character.to_string(),
            agent_kind: Some(agent.kind.clone()),
        });
        Self {
            id: session.id.to_string(),
            name: session.name.clone(),
            cwd: session.cwd.display().to_string(),
            status: session.status.as_str().to_owned(),
            status_since: session.status_since.millis(),
            started_at: session.started_at.millis(),
            focusable: session.origin == Origin::Registry && session.pid.is_some(),
            room: scene.cast.room.to_string(),
            actors: std::iter::once(lead).chain(agents).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum FocusDto {
    Tab { app: String },
    App { app: String },
    NotFound,
}

impl From<FocusOutcome> for FocusDto {
    fn from(outcome: FocusOutcome) -> Self {
        match outcome {
            FocusOutcome::Tab { app } => Self::Tab { app },
            FocusOutcome::App { app } => Self::App { app },
            FocusOutcome::NotFound => Self::NotFound,
        }
    }
}

pub fn hook_setup_name(setup: HookSetup) -> &'static str {
    match setup {
        HookSetup::Installed => "installed",
        HookSetup::Missing => "missing",
        HookSetup::Outdated => "outdated",
    }
}
