#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaVersion(u16);

impl SchemaVersion {
    pub const SUPPORTED: Self = Self(2);

    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickRateHz(u16);

impl TickRateHz {
    pub const fn new(value: u16) -> Result<Self, ScenarioValidationError> {
        if value == 0 {
            return Err(ScenarioValidationError::InvalidTickRate { found_hz: value });
        }

        Ok(Self(value))
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seed(u64);

impl Seed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunTicks(u32);

impl RunTicks {
    pub const fn new(value: u32) -> Result<Self, ScenarioValidationError> {
        if value == 0 {
            return Err(ScenarioValidationError::InvalidRunTicks { found: value });
        }

        Ok(Self(value))
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandSequence(u64);

impl CommandSequence {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetTick(u64);

impl TargetTick {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

macro_rules! identifier_type {
    ($name:ident, $kind:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdentifierValidationError> {
                let value = value.into();
                if value.is_empty() {
                    return Err(IdentifierValidationError::Empty { kind: $kind });
                }

                Ok(Self(value))
            }

            pub fn value(&self) -> &str {
                &self.0
            }
        }
    };
}

identifier_type!(MatchId, IdentifierKind::Match);
identifier_type!(TeamId, IdentifierKind::Team);
identifier_type!(ParticipantId, IdentifierKind::Participant);
identifier_type!(GroupId, IdentifierKind::Group);
identifier_type!(RobotId, IdentifierKind::Robot);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierKind {
    Match,
    Team,
    Participant,
    Group,
    Robot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentifierValidationError {
    Empty { kind: IdentifierKind },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamConfig {
    team_id: TeamId,
}

impl TeamConfig {
    pub fn new(team_id: TeamId) -> Self {
        Self { team_id }
    }

    pub fn team_id(&self) -> &TeamId {
        &self.team_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantConfig {
    participant_id: ParticipantId,
    team_id: TeamId,
}

impl ParticipantConfig {
    pub fn new(participant_id: ParticipantId, team_id: TeamId) -> Self {
        Self {
            participant_id,
            team_id,
        }
    }

    pub fn participant_id(&self) -> &ParticipantId {
        &self.participant_id
    }

    pub fn team_id(&self) -> &TeamId {
        &self.team_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupConfig {
    group_id: GroupId,
    controller_participant_id: ParticipantId,
    robot_ids: Vec<RobotId>,
}

impl GroupConfig {
    pub fn new(
        group_id: GroupId,
        controller_participant_id: ParticipantId,
        robot_ids: Vec<RobotId>,
    ) -> Self {
        Self {
            group_id,
            controller_participant_id,
            robot_ids,
        }
    }

    pub fn group_id(&self) -> &GroupId {
        &self.group_id
    }

    pub fn controller_participant_id(&self) -> &ParticipantId {
        &self.controller_participant_id
    }

    pub fn robot_ids(&self) -> &[RobotId] {
        &self.robot_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchConfig {
    match_id: MatchId,
    tick_rate_hz: TickRateHz,
    seed: Seed,
    teams: Vec<TeamConfig>,
    participants: Vec<ParticipantConfig>,
    groups: Vec<GroupConfig>,
}

impl MatchConfig {
    pub fn new(
        match_id: MatchId,
        tick_rate_hz: TickRateHz,
        seed: Seed,
        teams: Vec<TeamConfig>,
        participants: Vec<ParticipantConfig>,
        groups: Vec<GroupConfig>,
    ) -> Result<Self, MatchConfigValidationError> {
        validate_match_roster(&teams, &participants, &groups)?;

        Ok(Self {
            match_id,
            tick_rate_hz,
            seed,
            teams,
            participants,
            groups,
        })
    }

    pub fn match_id(&self) -> &MatchId {
        &self.match_id
    }

    pub const fn tick_rate_hz(&self) -> u16 {
        self.tick_rate_hz.value()
    }

    pub const fn seed(&self) -> u64 {
        self.seed.value()
    }

    pub fn teams(&self) -> &[TeamConfig] {
        &self.teams
    }

    pub fn participants(&self) -> &[ParticipantConfig] {
        &self.participants
    }

    pub fn groups(&self) -> &[GroupConfig] {
        &self.groups
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchConfigValidationError {
    DuplicateTeamId {
        team_id: TeamId,
    },
    DuplicateParticipantId {
        participant_id: ParticipantId,
    },
    UnknownParticipantTeam {
        participant_id: ParticipantId,
        team_id: TeamId,
    },
    DuplicateGroupId {
        group_id: GroupId,
    },
    UnknownGroupController {
        group_id: GroupId,
        controller_participant_id: ParticipantId,
    },
    DuplicateRobotId {
        robot_id: RobotId,
    },
}

fn validate_match_roster(
    teams: &[TeamConfig],
    participants: &[ParticipantConfig],
    groups: &[GroupConfig],
) -> Result<(), MatchConfigValidationError> {
    let mut team_ids = BTreeSet::new();
    for team in teams {
        if !team_ids.insert(team.team_id().clone()) {
            return Err(MatchConfigValidationError::DuplicateTeamId {
                team_id: team.team_id().clone(),
            });
        }
    }

    let mut participant_ids = BTreeSet::new();
    for participant in participants {
        if !participant_ids.insert(participant.participant_id().clone()) {
            return Err(MatchConfigValidationError::DuplicateParticipantId {
                participant_id: participant.participant_id().clone(),
            });
        }
        if !team_ids.contains(participant.team_id()) {
            return Err(MatchConfigValidationError::UnknownParticipantTeam {
                participant_id: participant.participant_id().clone(),
                team_id: participant.team_id().clone(),
            });
        }
    }

    let mut group_ids = BTreeSet::new();
    let mut robot_ids = BTreeSet::new();
    for group in groups {
        if !group_ids.insert(group.group_id().clone()) {
            return Err(MatchConfigValidationError::DuplicateGroupId {
                group_id: group.group_id().clone(),
            });
        }
        if !participant_ids.contains(group.controller_participant_id()) {
            return Err(MatchConfigValidationError::UnknownGroupController {
                group_id: group.group_id().clone(),
                controller_participant_id: group.controller_participant_id().clone(),
            });
        }
        for robot_id in group.robot_ids() {
            if !robot_ids.insert(robot_id.clone()) {
                return Err(MatchConfigValidationError::DuplicateRobotId {
                    robot_id: robot_id.clone(),
                });
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupOrder {
    HoldPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueGroupOrder {
    group_id: GroupId,
    order: GroupOrder,
}

impl IssueGroupOrder {
    pub fn new(group_id: GroupId, order: GroupOrder) -> Self {
        Self { group_id, order }
    }

    pub fn group_id(&self) -> &GroupId {
        &self.group_id
    }

    pub const fn order(&self) -> GroupOrder {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPayload {
    IssueGroupOrder(IssueGroupOrder),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEnvelope {
    sequence: CommandSequence,
    target_tick: TargetTick,
    participant_id: ParticipantId,
    payload: CommandPayload,
}

impl CommandEnvelope {
    pub fn new(
        sequence: CommandSequence,
        target_tick: TargetTick,
        participant_id: ParticipantId,
        payload: CommandPayload,
    ) -> Self {
        Self {
            sequence,
            target_tick,
            participant_id,
            payload,
        }
    }

    pub const fn sequence(&self) -> CommandSequence {
        self.sequence
    }

    pub const fn target_tick(&self) -> TargetTick {
        self.target_tick
    }

    pub fn participant_id(&self) -> &ParticipantId {
        &self.participant_id
    }

    pub const fn payload(&self) -> &CommandPayload {
        &self.payload
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandRejectionReason {
    WrongTargetTick,
    UnknownParticipant,
    UnknownGroup,
    GroupNotControlledByParticipant,
    DuplicateCommandSequence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandResultStatus {
    Accepted,
    Rejected { reason: CommandRejectionReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    sequence: CommandSequence,
    target_tick: TargetTick,
    participant_id: ParticipantId,
    status: CommandResultStatus,
}

impl CommandResult {
    pub fn new(
        sequence: CommandSequence,
        target_tick: TargetTick,
        participant_id: ParticipantId,
        status: CommandResultStatus,
    ) -> Self {
        Self {
            sequence,
            target_tick,
            participant_id,
            status,
        }
    }

    pub const fn sequence(&self) -> CommandSequence {
        self.sequence
    }

    pub const fn target_tick(&self) -> TargetTick {
        self.target_tick
    }

    pub fn participant_id(&self) -> &ParticipantId {
        &self.participant_id
    }

    pub const fn status(&self) -> CommandResultStatus {
        self.status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventOrdinal(u32);

impl EventOrdinal {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupOrderAssignedEvent {
    tick: u64,
    ordinal: EventOrdinal,
    group_id: GroupId,
    participant_id: ParticipantId,
    order: GroupOrder,
}

impl GroupOrderAssignedEvent {
    pub fn new(
        tick: u64,
        ordinal: EventOrdinal,
        group_id: GroupId,
        participant_id: ParticipantId,
        order: GroupOrder,
    ) -> Self {
        Self {
            tick,
            ordinal,
            group_id,
            participant_id,
            order,
        }
    }

    pub const fn tick(&self) -> u64 {
        self.tick
    }

    pub const fn ordinal(&self) -> EventOrdinal {
        self.ordinal
    }

    pub fn group_id(&self) -> &GroupId {
        &self.group_id
    }

    pub fn participant_id(&self) -> &ParticipantId {
        &self.participant_id
    }

    pub const fn order(&self) -> GroupOrder {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameplayEvent {
    GroupOrderAssigned(GroupOrderAssignedEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessScenario {
    schema_version: SchemaVersion,
    match_config: MatchConfig,
    commands: Vec<CommandEnvelope>,
    run_ticks: RunTicks,
    trace: bool,
}

impl HeadlessScenario {
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    pub const fn match_config(&self) -> &MatchConfig {
        &self.match_config
    }

    pub fn commands(&self) -> &[CommandEnvelope] {
        &self.commands
    }

    pub const fn run_ticks(&self) -> u32 {
        self.run_ticks.value()
    }

    pub const fn trace_enabled(&self) -> bool {
        self.trace
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioInputError {
    DataShape { message: String },
    Validation(ScenarioValidationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioValidationError {
    UnsupportedSchemaVersion {
        found: SchemaVersion,
        supported: SchemaVersion,
    },
    InvalidTickRate {
        found_hz: u16,
    },
    InvalidRunTicks {
        found: u32,
    },
    InvalidIdentifier {
        error: IdentifierValidationError,
    },
    InvalidMatchConfig {
        error: MatchConfigValidationError,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawHeadlessScenario {
    schema_version: u16,
    #[serde(rename = "match")]
    match_config: RawMatchConfig,
    commands: Vec<RawCommandEnvelope>,
    run_ticks: u32,
    trace: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawMatchConfig {
    match_id: String,
    tick_rate_hz: u16,
    seed: u64,
    teams: Vec<RawTeamConfig>,
    participants: Vec<RawParticipantConfig>,
    groups: Vec<RawGroupConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawTeamConfig {
    team_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawParticipantConfig {
    participant_id: String,
    team_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawGroupConfig {
    group_id: String,
    controller_participant_id: String,
    robot_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawCommandEnvelope {
    sequence: u64,
    target_tick: u64,
    participant_id: String,
    payload: RawCommandPayload,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "PascalCase", deny_unknown_fields)]
enum RawCommandPayload {
    IssueGroupOrder {
        #[serde(rename = "groupId")]
        group_id: String,
        order: RawGroupOrder,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "PascalCase", deny_unknown_fields)]
enum RawGroupOrder {
    HoldPosition,
}

pub fn parse_headless_scenario_json(source: &str) -> Result<HeadlessScenario, ScenarioInputError> {
    let raw = serde_json::from_str::<RawHeadlessScenario>(source).map_err(|error| {
        ScenarioInputError::DataShape {
            message: error.to_string(),
        }
    })?;

    validate_headless_scenario(raw).map_err(ScenarioInputError::Validation)
}

fn validate_headless_scenario(
    raw: RawHeadlessScenario,
) -> Result<HeadlessScenario, ScenarioValidationError> {
    let schema_version = SchemaVersion::new(raw.schema_version);
    if schema_version != SchemaVersion::SUPPORTED {
        return Err(ScenarioValidationError::UnsupportedSchemaVersion {
            found: schema_version,
            supported: SchemaVersion::SUPPORTED,
        });
    }

    let match_id = MatchId::new(raw.match_config.match_id)
        .map_err(|error| ScenarioValidationError::InvalidIdentifier { error })?;
    let tick_rate_hz = TickRateHz::new(raw.match_config.tick_rate_hz)?;
    let run_ticks = RunTicks::new(raw.run_ticks)?;
    let teams = raw
        .match_config
        .teams
        .into_iter()
        .map(|team| {
            Ok(TeamConfig::new(TeamId::new(team.team_id).map_err(
                |error| ScenarioValidationError::InvalidIdentifier { error },
            )?))
        })
        .collect::<Result<Vec<_>, ScenarioValidationError>>()?;
    let participants = raw
        .match_config
        .participants
        .into_iter()
        .map(|participant| {
            Ok(ParticipantConfig::new(
                ParticipantId::new(participant.participant_id)
                    .map_err(|error| ScenarioValidationError::InvalidIdentifier { error })?,
                TeamId::new(participant.team_id)
                    .map_err(|error| ScenarioValidationError::InvalidIdentifier { error })?,
            ))
        })
        .collect::<Result<Vec<_>, ScenarioValidationError>>()?;
    let groups = raw
        .match_config
        .groups
        .into_iter()
        .map(|group| {
            Ok(GroupConfig::new(
                GroupId::new(group.group_id)
                    .map_err(|error| ScenarioValidationError::InvalidIdentifier { error })?,
                ParticipantId::new(group.controller_participant_id)
                    .map_err(|error| ScenarioValidationError::InvalidIdentifier { error })?,
                group
                    .robot_ids
                    .into_iter()
                    .map(|robot_id| {
                        RobotId::new(robot_id)
                            .map_err(|error| ScenarioValidationError::InvalidIdentifier { error })
                    })
                    .collect::<Result<Vec<_>, ScenarioValidationError>>()?,
            ))
        })
        .collect::<Result<Vec<_>, ScenarioValidationError>>()?;
    let match_config = MatchConfig::new(
        match_id,
        tick_rate_hz,
        Seed::new(raw.match_config.seed),
        teams,
        participants,
        groups,
    )
    .map_err(|error| ScenarioValidationError::InvalidMatchConfig { error })?;
    let commands = raw
        .commands
        .into_iter()
        .map(|command| {
            Ok(CommandEnvelope::new(
                CommandSequence::new(command.sequence),
                TargetTick::new(command.target_tick),
                ParticipantId::new(command.participant_id)
                    .map_err(|error| ScenarioValidationError::InvalidIdentifier { error })?,
                match command.payload {
                    RawCommandPayload::IssueGroupOrder { group_id, order } => {
                        CommandPayload::IssueGroupOrder(IssueGroupOrder::new(
                            GroupId::new(group_id).map_err(|error| {
                                ScenarioValidationError::InvalidIdentifier { error }
                            })?,
                            match order {
                                RawGroupOrder::HoldPosition => GroupOrder::HoldPosition,
                            },
                        ))
                    }
                },
            ))
        })
        .collect::<Result<Vec<_>, ScenarioValidationError>>()?;

    Ok(HeadlessScenario {
        schema_version,
        match_config,
        commands,
        run_ticks,
        trace: raw.trace,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id_match(value: &str) -> MatchId {
        MatchId::new(value).expect("test match id is valid")
    }

    fn id_team(value: &str) -> TeamId {
        TeamId::new(value).expect("test team id is valid")
    }

    fn id_participant(value: &str) -> ParticipantId {
        ParticipantId::new(value).expect("test participant id is valid")
    }

    fn id_group(value: &str) -> GroupId {
        GroupId::new(value).expect("test group id is valid")
    }

    fn id_robot(value: &str) -> RobotId {
        RobotId::new(value).expect("test robot id is valid")
    }

    fn valid_roster_match_config() -> MatchConfig {
        MatchConfig::new(
            id_match("match-001"),
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(123456),
            vec![
                TeamConfig::new(id_team("team-blue")),
                TeamConfig::new(id_team("team-red")),
            ],
            vec![
                ParticipantConfig::new(id_participant("participant-blue-1"), id_team("team-blue")),
                ParticipantConfig::new(id_participant("participant-red-1"), id_team("team-red")),
            ],
            vec![
                GroupConfig::new(
                    id_group("group-blue-alpha"),
                    id_participant("participant-blue-1"),
                    vec![id_robot("robot-blue-001"), id_robot("robot-blue-002")],
                ),
                GroupConfig::new(
                    id_group("group-red-alpha"),
                    id_participant("participant-red-1"),
                    vec![id_robot("robot-red-001"), id_robot("robot-red-002")],
                ),
            ],
        )
        .expect("valid roster should pass validation")
    }

    fn valid_scenario_json(seed: u64) -> String {
        format!(
            r#"{{
                "schemaVersion": 2,
                "match": {{
                    "matchId": "match-001",
                    "tickRateHz": 20,
                    "seed": {seed},
                    "teams": [],
                    "participants": [],
                    "groups": []
                }},
                "commands": [],
                "runTicks": 3,
                "trace": true
            }}"#
        )
    }

    #[test]
    fn valid_scenario_is_accepted() {
        let scenario = parse_headless_scenario_json(&valid_scenario_json(123456))
            .expect("valid scenario should pass validation");

        assert_eq!(scenario.schema_version(), SchemaVersion::SUPPORTED);
        assert_eq!(scenario.match_config().match_id().value(), "match-001");
        assert_eq!(scenario.match_config().tick_rate_hz(), 20);
        assert_eq!(scenario.run_ticks(), 3);
        assert!(scenario.trace_enabled());
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let json =
            valid_scenario_json(123456).replace(r#""schemaVersion": 2"#, r#""schemaVersion": 1"#);

        let error = parse_headless_scenario_json(&json).expect_err("version 1 is unsupported");

        assert_eq!(
            error,
            ScenarioInputError::Validation(ScenarioValidationError::UnsupportedSchemaVersion {
                found: SchemaVersion::new(1),
                supported: SchemaVersion::SUPPORTED,
            })
        );
    }

    #[test]
    fn invalid_tick_rate_is_rejected() {
        let json = valid_scenario_json(123456).replace(r#""tickRateHz": 20"#, r#""tickRateHz": 0"#);

        let error = parse_headless_scenario_json(&json).expect_err("zero tick rate is invalid");

        assert_eq!(
            error,
            ScenarioInputError::Validation(ScenarioValidationError::InvalidTickRate {
                found_hz: 0,
            })
        );
    }

    #[test]
    fn invalid_tick_count_is_rejected() {
        let json = valid_scenario_json(123456).replace(r#""runTicks": 3"#, r#""runTicks": 0"#);

        let error = parse_headless_scenario_json(&json).expect_err("zero tick count is invalid");

        assert_eq!(
            error,
            ScenarioInputError::Validation(ScenarioValidationError::InvalidRunTicks { found: 0 })
        );
    }

    #[test]
    fn seed_is_preserved_without_modification() {
        let seed = u64::MAX;

        let scenario = parse_headless_scenario_json(&valid_scenario_json(seed))
            .expect("maximum seed should pass validation");

        assert_eq!(scenario.match_config().seed(), seed);
    }

    #[test]
    fn valid_roster_with_teams_participants_and_groups_is_accepted() {
        let config = valid_roster_match_config();

        assert_eq!(config.match_id().value(), "match-001");
        assert_eq!(config.teams().len(), 2);
        assert_eq!(config.participants().len(), 2);
        assert_eq!(config.groups().len(), 2);
    }

    #[test]
    fn participant_referencing_unknown_team_is_rejected() {
        let error = MatchConfig::new(
            id_match("match-001"),
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(123456),
            vec![TeamConfig::new(id_team("team-blue"))],
            vec![ParticipantConfig::new(
                id_participant("participant-red-1"),
                id_team("team-red"),
            )],
            vec![],
        )
        .expect_err("participant cannot reference an unknown team");

        assert_eq!(
            error,
            MatchConfigValidationError::UnknownParticipantTeam {
                participant_id: id_participant("participant-red-1"),
                team_id: id_team("team-red"),
            }
        );
    }

    #[test]
    fn group_controlled_by_unknown_participant_is_rejected() {
        let error = MatchConfig::new(
            id_match("match-001"),
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(123456),
            vec![TeamConfig::new(id_team("team-blue"))],
            vec![ParticipantConfig::new(
                id_participant("participant-blue-1"),
                id_team("team-blue"),
            )],
            vec![GroupConfig::new(
                id_group("group-blue-alpha"),
                id_participant("participant-red-1"),
                vec![],
            )],
        )
        .expect_err("group cannot reference an unknown controlling participant");

        assert_eq!(
            error,
            MatchConfigValidationError::UnknownGroupController {
                group_id: id_group("group-blue-alpha"),
                controller_participant_id: id_participant("participant-red-1"),
            }
        );
    }

    #[test]
    fn duplicate_roster_identifiers_are_rejected() {
        let duplicate_team = MatchConfig::new(
            id_match("match-001"),
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(123456),
            vec![
                TeamConfig::new(id_team("team-blue")),
                TeamConfig::new(id_team("team-blue")),
            ],
            vec![],
            vec![],
        )
        .expect_err("duplicate teams should be rejected");

        assert_eq!(
            duplicate_team,
            MatchConfigValidationError::DuplicateTeamId {
                team_id: id_team("team-blue"),
            }
        );

        let duplicate_participant = MatchConfig::new(
            id_match("match-001"),
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(123456),
            vec![TeamConfig::new(id_team("team-blue"))],
            vec![
                ParticipantConfig::new(id_participant("participant-blue-1"), id_team("team-blue")),
                ParticipantConfig::new(id_participant("participant-blue-1"), id_team("team-blue")),
            ],
            vec![],
        )
        .expect_err("duplicate participants should be rejected");

        assert_eq!(
            duplicate_participant,
            MatchConfigValidationError::DuplicateParticipantId {
                participant_id: id_participant("participant-blue-1"),
            }
        );

        let duplicate_group = MatchConfig::new(
            id_match("match-001"),
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(123456),
            vec![TeamConfig::new(id_team("team-blue"))],
            vec![ParticipantConfig::new(
                id_participant("participant-blue-1"),
                id_team("team-blue"),
            )],
            vec![
                GroupConfig::new(
                    id_group("group-blue-alpha"),
                    id_participant("participant-blue-1"),
                    vec![],
                ),
                GroupConfig::new(
                    id_group("group-blue-alpha"),
                    id_participant("participant-blue-1"),
                    vec![],
                ),
            ],
        )
        .expect_err("duplicate groups should be rejected");

        assert_eq!(
            duplicate_group,
            MatchConfigValidationError::DuplicateGroupId {
                group_id: id_group("group-blue-alpha"),
            }
        );
    }

    #[test]
    fn robot_assigned_to_more_than_one_group_is_rejected() {
        let error = MatchConfig::new(
            id_match("match-001"),
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(123456),
            vec![TeamConfig::new(id_team("team-blue"))],
            vec![ParticipantConfig::new(
                id_participant("participant-blue-1"),
                id_team("team-blue"),
            )],
            vec![
                GroupConfig::new(
                    id_group("group-blue-alpha"),
                    id_participant("participant-blue-1"),
                    vec![id_robot("robot-blue-001")],
                ),
                GroupConfig::new(
                    id_group("group-blue-beta"),
                    id_participant("participant-blue-1"),
                    vec![id_robot("robot-blue-001")],
                ),
            ],
        )
        .expect_err("a robot cannot be assigned to multiple groups");

        assert_eq!(
            error,
            MatchConfigValidationError::DuplicateRobotId {
                robot_id: id_robot("robot-blue-001"),
            }
        );
    }
}
