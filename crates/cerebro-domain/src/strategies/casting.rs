//! Stable casting: which room and which characters each session gets.
//!
//! Rules:
//! 1. A pick is kept for as long as its session or agent is alive (stable on screen).
//! 2. New picks are pseudo-random but deterministic (seeded by the id).
//! 3. Live sessions avoid sharing a room; characters are unique within a room.
//! 4. Older sessions are served first, so a newcomer never steals an existing room.

use std::collections::HashSet;

use crate::entities::{ActorKey, Assignments, Cast, Catalog, CharacterId, RoomId, Session};
use crate::rng::pick_free;

/// Returns one `Cast` per session (same order) and whether `assignments` changed.
pub fn cast_all(
    catalog: &Catalog,
    assignments: &mut Assignments,
    sessions: &[Session],
) -> (Vec<Cast>, bool) {
    let before = assignments.clone();
    forget_dead(assignments, sessions);

    let mut order: Vec<usize> = (0..sessions.len()).collect();
    order.sort_by(|&left_index, &right_index| {
        let (left, right) = (&sessions[left_index], &sessions[right_index]);
        left.started_at
            .cmp(&right.started_at)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut casts: Vec<Option<Cast>> = vec![None; sessions.len()];
    let mut rooms_taken: HashSet<RoomId> = HashSet::new();
    for index in order {
        let session = &sessions[index];
        let room = pick_room(catalog, assignments, session, &rooms_taken);
        rooms_taken.insert(room.clone());

        let mut in_room: HashSet<CharacterId> = HashSet::new();
        let mut cast_one = |key: ActorKey| {
            let character = pick_character(catalog, assignments, &key, &in_room);
            in_room.insert(character.clone());
            assignments.characters.insert(key, character.clone());
            character
        };
        let lead = cast_one(ActorKey::Lead(session.id.clone()));
        let agents = session
            .agents
            .iter()
            .map(|agent| cast_one(ActorKey::Agent(agent.id.clone())))
            .collect();

        assignments.rooms.insert(session.id.clone(), room.clone());
        casts[index] = Some(Cast { room, lead, agents });
    }

    let changed = *assignments != before;
    (casts.into_iter().flatten().collect(), changed)
}

fn forget_dead(assignments: &mut Assignments, sessions: &[Session]) {
    let live_sessions: HashSet<_> = sessions.iter().map(|session| &session.id).collect();
    let live_agents: HashSet<_> = sessions
        .iter()
        .flat_map(|session| session.agents.iter().map(|agent| &agent.id))
        .collect();
    assignments.rooms.retain(|id, _| live_sessions.contains(id));
    assignments.characters.retain(|key, _| match key {
        ActorKey::Lead(id) => live_sessions.contains(id),
        ActorKey::Agent(id) => live_agents.contains(id),
    });
}

fn pick_room(
    catalog: &Catalog,
    assignments: &Assignments,
    session: &Session,
    taken: &HashSet<RoomId>,
) -> RoomId {
    if let Some(room) = assignments.rooms.get(&session.id)
        && catalog.rooms().contains(room)
        && !taken.contains(room)
    {
        return room.clone();
    }
    let rooms = catalog.rooms();
    let seed = format!("room:{}", session.id);
    let index = pick_free(&seed, rooms.len(), |room_index| {
        taken.contains(&rooms[room_index])
    })
    .unwrap_or(0);
    rooms[index].clone()
}

fn pick_character(
    catalog: &Catalog,
    assignments: &Assignments,
    key: &ActorKey,
    taken: &HashSet<CharacterId>,
) -> CharacterId {
    if let Some(character) = assignments.characters.get(key)
        && catalog.characters().contains(character)
        && !taken.contains(character)
    {
        return character.clone();
    }
    let characters = catalog.characters();
    let index = pick_free(&key.encode(), characters.len(), |character_index| {
        taken.contains(&characters[character_index])
    })
    .unwrap_or(0);
    characters[index].clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Agent, AgentId, Origin, SessionId, SessionStatus};
    use crate::time::Timestamp;

    fn catalog(characters: usize, rooms: usize) -> Catalog {
        Catalog::new(
            (0..characters)
                .map(|index| CharacterId::new(format!("c{index}")))
                .collect(),
            (0..rooms)
                .map(|index| RoomId::new(format!("r{index}")))
                .collect(),
        )
        .unwrap()
    }

    fn session(id: &str, started: u64, agents: &[&str]) -> Session {
        Session {
            id: SessionId::new(id),
            pid: Some(1),
            name: id.into(),
            cwd: "/".into(),
            status: SessionStatus::Busy,
            started_at: Timestamp(started),
            status_since: Timestamp(started),
            origin: Origin::Registry,
            agents: agents
                .iter()
                .map(|agent_id| Agent {
                    id: AgentId::new(*agent_id),
                    kind: "Explore".into(),
                    started_at: Timestamp(started),
                })
                .collect(),
        }
    }

    #[test]
    fn rooms_are_unique_while_available() {
        let catalog = catalog(10, 3);
        let sessions: Vec<_> = (0..3)
            .map(|index| session(&format!("s{index}"), index, &[]))
            .collect();
        let (casts, _) = cast_all(&catalog, &mut Assignments::default(), &sessions);
        let rooms: HashSet<_> = casts.iter().map(|cast| &cast.room).collect();
        assert_eq!(rooms.len(), 3);
    }

    #[test]
    fn characters_are_unique_within_a_room() {
        let catalog = catalog(4, 1);
        let sessions = [session("s", 0, &["a", "b", "c"])];
        let (casts, _) = cast_all(&catalog, &mut Assignments::default(), &sessions);
        let mut all = vec![casts[0].lead.clone()];
        all.extend(casts[0].agents.clone());
        assert_eq!(all.iter().collect::<HashSet<_>>().len(), 4);
    }

    #[test]
    fn picks_survive_when_another_actor_leaves() {
        // Regression for report item m3: removing an agent must not reshuffle the others.
        let catalog = catalog(3, 3);
        let mut memory = Assignments::default();
        let (before, _) = cast_all(&catalog, &mut memory, &[session("s", 0, &["a", "b"])]);
        let (after, _) = cast_all(&catalog, &mut memory, &[session("s", 0, &["b"])]);
        assert_eq!(after[0].lead, before[0].lead);
        assert_eq!(after[0].agents[0], before[0].agents[1]);
        assert_eq!(after[0].room, before[0].room);
    }

    #[test]
    fn older_session_keeps_its_room_when_a_newer_one_arrives() {
        let catalog = catalog(5, 5);
        let mut memory = Assignments::default();
        let (first, _) = cast_all(&catalog, &mut memory, &[session("old", 0, &[])]);
        let (second, _) = cast_all(
            &catalog,
            &mut memory,
            &[session("new", 5, &[]), session("old", 0, &[])],
        );
        assert_eq!(second[1].room, first[0].room);
        assert_ne!(second[0].room, second[1].room);
    }

    #[test]
    fn dead_entries_are_forgotten_and_change_is_reported() {
        let catalog = catalog(5, 5);
        let mut memory = Assignments::default();
        let (_, changed) = cast_all(&catalog, &mut memory, &[session("s", 0, &["a"])]);
        assert!(changed);
        let (_, changed) = cast_all(&catalog, &mut memory, &[session("s", 0, &["a"])]);
        assert!(!changed);
        cast_all(&catalog, &mut memory, &[]);
        assert!(memory.rooms.is_empty() && memory.characters.is_empty());
    }

    #[test]
    fn more_sessions_than_rooms_still_works() {
        let catalog = catalog(5, 1);
        let sessions = [session("a", 0, &[]), session("b", 1, &[])];
        let (casts, _) = cast_all(&catalog, &mut Assignments::default(), &sessions);
        assert_eq!(casts.len(), 2);
    }
}
