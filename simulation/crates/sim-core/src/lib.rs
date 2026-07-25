#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

pub use sim_protocol::{CommandRejectionReason, CommandResult, CommandResultStatus};

use sim_protocol::{
    CommandEnvelope, CommandPayload, CommandSequence, EventOrdinal, GameplayEvent, GroupId,
    GroupOrder, GroupOrderAssignedEvent, MatchConfig, ParticipantId,
};

const AUTHORITATIVE_STATE_VERSION: u16 = 1;
const STATE_HASH_ALGORITHM: &str = "BLAKE3-256";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchCreationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickExecutionError {
    TickLimitReached { current_tick: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickResult {
    started_tick: u64,
    completed_tick: u64,
    state_hash: StateHash,
    command_results: Vec<CommandResult>,
    gameplay_events: Vec<GameplayEvent>,
}

impl TickResult {
    pub const fn started_tick(&self) -> u64 {
        self.started_tick
    }

    pub const fn completed_tick(&self) -> u64 {
        self.completed_tick
    }

    pub const fn state_hash(&self) -> StateHash {
        self.state_hash
    }

    pub fn command_results(&self) -> &[CommandResult] {
        &self.command_results
    }

    pub fn gameplay_events(&self) -> &[GameplayEvent] {
        &self.gameplay_events
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateHash([u8; 32]);

impl StateHash {
    pub const fn algorithm(self) -> &'static str {
        STATE_HASH_ALGORITHM
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceRecordKind {
    TickStarted,
    CommandsSelected,
    CommandsNormalized,
    CommandValidationCompleted,
    IntentProduced,
    IntentApplied,
    GameplayEventsFinalized,
    TickTransitionCalculated,
    TickTransitionApplied,
    StateHashCalculated,
    TickCompleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceRecord {
    tick: u64,
    kind: TraceRecordKind,
}

impl TraceRecord {
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    pub const fn kind(&self) -> TraceRecordKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionTrace {
    records: Vec<TraceRecord>,
}

impl ExecutionTrace {
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn records(&self) -> &[TraceRecord] {
        &self.records
    }

    fn record(&mut self, tick: AuthoritativeTick, kind: TraceRecordKind) {
        self.records.push(TraceRecord {
            tick: tick.value(),
            kind,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchSimulation {
    state: AuthoritativeState,
}

impl MatchSimulation {
    pub fn new(config: MatchConfig) -> Result<Self, MatchCreationError> {
        Ok(Self {
            state: AuthoritativeState::new(config),
        })
    }

    pub const fn current_tick(&self) -> u64 {
        self.state.current_tick.value()
    }

    pub const fn match_config(&self) -> &MatchConfig {
        &self.state.config
    }

    pub fn state_hash(&self) -> StateHash {
        self.state.hash()
    }

    pub fn active_group_order(&self, group_id: &GroupId) -> Option<GroupOrder> {
        self.state.active_group_orders.get(group_id).copied()
    }

    pub fn validate_command(&self, command: &CommandEnvelope) -> CommandResult {
        self.validate_command_for_intent(command).result
    }

    pub fn execute_tick(&mut self) -> Result<TickResult, TickExecutionError> {
        self.execute_tick_with_commands(&[])
    }

    pub fn execute_tick_with_trace(
        &mut self,
        trace: Option<&mut ExecutionTrace>,
    ) -> Result<TickResult, TickExecutionError> {
        self.execute_tick_with_commands_and_trace(&[], trace)
    }

    pub fn execute_tick_with_commands(
        &mut self,
        commands: &[CommandEnvelope],
    ) -> Result<TickResult, TickExecutionError> {
        self.execute_tick_with_commands_and_trace(commands, None)
    }

    pub fn execute_tick_with_commands_and_trace(
        &mut self,
        commands: &[CommandEnvelope],
        trace: Option<&mut ExecutionTrace>,
    ) -> Result<TickResult, TickExecutionError> {
        let mut trace = trace;
        let started_tick = self.verify_next_tick_may_begin()?;
        record_trace(&mut trace, started_tick, TraceRecordKind::TickStarted);

        let selected_commands = select_commands_for_tick(commands);
        record_trace(&mut trace, started_tick, TraceRecordKind::CommandsSelected);

        let normalized_commands = normalize_commands(selected_commands);
        record_trace(
            &mut trace,
            started_tick,
            TraceRecordKind::CommandsNormalized,
        );

        let duplicate_sequences = duplicate_sequences(&normalized_commands);
        let mut command_results = Vec::with_capacity(normalized_commands.len());
        let mut accepted_intents = Vec::new();

        for command in normalized_commands {
            let outcome = if duplicate_sequences.contains(&command.sequence()) {
                self.reject_command(command, CommandRejectionReason::DuplicateCommandSequence)
            } else {
                self.validate_command_for_intent(command)
            };
            record_trace(
                &mut trace,
                started_tick,
                TraceRecordKind::CommandValidationCompleted,
            );

            if let Some(intent) = outcome.intent {
                record_trace(&mut trace, started_tick, TraceRecordKind::IntentProduced);
                accepted_intents.push((command.participant_id().clone(), intent));
            }

            command_results.push(outcome.result);
        }

        let mut gameplay_events = Vec::with_capacity(accepted_intents.len());
        for (participant_id, intent) in accepted_intents {
            let event =
                self.apply_intent(started_tick, participant_id, intent, gameplay_events.len());
            record_trace(&mut trace, started_tick, TraceRecordKind::IntentApplied);
            gameplay_events.push(event);
        }
        record_trace(
            &mut trace,
            started_tick,
            TraceRecordKind::GameplayEventsFinalized,
        );

        let completed_tick = calculate_next_tick(started_tick)?;
        record_trace(
            &mut trace,
            started_tick,
            TraceRecordKind::TickTransitionCalculated,
        );

        self.apply_tick_transition(completed_tick);
        record_trace(
            &mut trace,
            completed_tick,
            TraceRecordKind::TickTransitionApplied,
        );

        let state_hash = self.state_hash();
        record_trace(
            &mut trace,
            completed_tick,
            TraceRecordKind::StateHashCalculated,
        );
        record_trace(&mut trace, completed_tick, TraceRecordKind::TickCompleted);

        Ok(TickResult {
            started_tick: started_tick.value(),
            completed_tick: completed_tick.value(),
            state_hash,
            command_results,
            gameplay_events,
        })
    }

    fn verify_next_tick_may_begin(&self) -> Result<AuthoritativeTick, TickExecutionError> {
        let current_tick = self.state.current_tick;
        if current_tick.value() == u64::MAX {
            return Err(TickExecutionError::TickLimitReached {
                current_tick: current_tick.value(),
            });
        }

        Ok(current_tick)
    }

    fn apply_tick_transition(&mut self, completed_tick: AuthoritativeTick) {
        self.state.current_tick = completed_tick;
    }

    fn validate_command_for_intent(&self, command: &CommandEnvelope) -> CommandValidationOutcome {
        if command.target_tick().value() != self.current_tick() {
            return self.reject_command(command, CommandRejectionReason::WrongTargetTick);
        }

        if !self
            .state
            .config
            .participants()
            .iter()
            .any(|participant| participant.participant_id() == command.participant_id())
        {
            return self.reject_command(command, CommandRejectionReason::UnknownParticipant);
        }

        match command.payload() {
            CommandPayload::IssueGroupOrder(command_payload) => {
                let Some(group) = self
                    .state
                    .config
                    .groups()
                    .iter()
                    .find(|group| group.group_id() == command_payload.group_id())
                else {
                    return self.reject_command(command, CommandRejectionReason::UnknownGroup);
                };

                if group.controller_participant_id() != command.participant_id() {
                    return self.reject_command(
                        command,
                        CommandRejectionReason::GroupNotControlledByParticipant,
                    );
                }

                self.accept_command(
                    command,
                    CommandIntent::IssueGroupOrder(GroupOrderIntent {
                        group_id: command_payload.group_id().clone(),
                        order: command_payload.order(),
                    }),
                )
            }
        }
    }

    fn accept_command(
        &self,
        command: &CommandEnvelope,
        intent: CommandIntent,
    ) -> CommandValidationOutcome {
        CommandValidationOutcome {
            result: command_validation_result(command, CommandResultStatus::Accepted),
            intent: Some(intent),
        }
    }

    fn reject_command(
        &self,
        command: &CommandEnvelope,
        reason: CommandRejectionReason,
    ) -> CommandValidationOutcome {
        CommandValidationOutcome {
            result: command_validation_result(command, CommandResultStatus::Rejected { reason }),
            intent: None,
        }
    }

    fn apply_intent(
        &mut self,
        tick: AuthoritativeTick,
        participant_id: ParticipantId,
        intent: CommandIntent,
        event_index: usize,
    ) -> GameplayEvent {
        match intent {
            CommandIntent::IssueGroupOrder(intent) => {
                self.state
                    .active_group_orders
                    .insert(intent.group_id.clone(), intent.order);

                GameplayEvent::GroupOrderAssigned(GroupOrderAssignedEvent::new(
                    tick.value(),
                    EventOrdinal::new(event_index as u32),
                    intent.group_id,
                    participant_id,
                    intent.order,
                ))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandValidationOutcome {
    result: CommandResult,
    intent: Option<CommandIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandIntent {
    IssueGroupOrder(GroupOrderIntent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupOrderIntent {
    group_id: GroupId,
    order: GroupOrder,
}

fn command_validation_result(
    command: &CommandEnvelope,
    status: CommandResultStatus,
) -> CommandResult {
    CommandResult::new(
        command.sequence(),
        command.target_tick(),
        command.participant_id().clone(),
        status,
    )
}

fn select_commands_for_tick(commands: &[CommandEnvelope]) -> Vec<&CommandEnvelope> {
    commands.iter().collect()
}

fn normalize_commands(mut commands: Vec<&CommandEnvelope>) -> Vec<&CommandEnvelope> {
    commands.sort_by(canonical_command_order);
    commands
}

fn canonical_command_order(
    left: &&CommandEnvelope,
    right: &&CommandEnvelope,
) -> std::cmp::Ordering {
    canonical_command_key(left).cmp(&canonical_command_key(right))
}

fn canonical_command_key(command: &CommandEnvelope) -> CommandSortKey {
    let (payload_kind, group_id, order) = match command.payload() {
        CommandPayload::IssueGroupOrder(payload) => (
            0_u8,
            payload.group_id().value().to_owned(),
            group_order_sort_value(payload.order()),
        ),
    };

    CommandSortKey {
        sequence: command.sequence().value(),
        target_tick: command.target_tick().value(),
        participant_id: command.participant_id().value().to_owned(),
        payload_kind,
        group_id,
        order,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CommandSortKey {
    sequence: u64,
    target_tick: u64,
    participant_id: String,
    payload_kind: u8,
    group_id: String,
    order: u8,
}

fn group_order_sort_value(order: GroupOrder) -> u8 {
    match order {
        GroupOrder::HoldPosition => 0,
    }
}

fn duplicate_sequences(commands: &[&CommandEnvelope]) -> BTreeSet<CommandSequence> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();

    for command in commands {
        if !seen.insert(command.sequence()) {
            duplicates.insert(command.sequence());
        }
    }

    duplicates
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthoritativeState {
    config: MatchConfig,
    current_tick: AuthoritativeTick,
    active_group_orders: BTreeMap<GroupId, GroupOrder>,
}

impl AuthoritativeState {
    fn new(config: MatchConfig) -> Self {
        Self {
            config,
            current_tick: AuthoritativeTick::ZERO,
            active_group_orders: BTreeMap::new(),
        }
    }

    fn hash(&self) -> StateHash {
        let mut canonical = Vec::new();
        canonical.extend_from_slice(&AUTHORITATIVE_STATE_VERSION.to_le_bytes());
        push_string(&mut canonical, self.config.match_id().value());
        canonical.extend_from_slice(&self.config.tick_rate_hz().to_le_bytes());
        canonical.extend_from_slice(&self.config.seed().to_le_bytes());
        canonical.extend_from_slice(&self.current_tick.value().to_le_bytes());
        push_roster(&mut canonical, &self.config);
        push_active_group_orders(&mut canonical, &self.active_group_orders);

        StateHash(*blake3::hash(&canonical).as_bytes())
    }
}

fn push_roster(canonical: &mut Vec<u8>, config: &MatchConfig) {
    let mut teams = config.teams().iter().collect::<Vec<_>>();
    teams.sort_by_key(|team| team.team_id().value());
    push_len(canonical, teams.len());
    for team in teams {
        push_string(canonical, team.team_id().value());
    }

    let mut participants = config.participants().iter().collect::<Vec<_>>();
    participants.sort_by_key(|participant| participant.participant_id().value());
    push_len(canonical, participants.len());
    for participant in participants {
        push_string(canonical, participant.participant_id().value());
        push_string(canonical, participant.team_id().value());
    }

    let mut groups = config.groups().iter().collect::<Vec<_>>();
    groups.sort_by_key(|group| group.group_id().value());
    push_len(canonical, groups.len());
    for group in groups {
        push_string(canonical, group.group_id().value());
        push_string(canonical, group.controller_participant_id().value());

        let mut robot_ids = group.robot_ids().iter().collect::<Vec<_>>();
        robot_ids.sort_by_key(|robot_id| robot_id.value());
        push_len(canonical, robot_ids.len());
        for robot_id in robot_ids {
            push_string(canonical, robot_id.value());
        }
    }
}

fn push_active_group_orders(
    canonical: &mut Vec<u8>,
    active_group_orders: &BTreeMap<GroupId, GroupOrder>,
) {
    push_len(canonical, active_group_orders.len());
    for (group_id, order) in active_group_orders {
        push_string(canonical, group_id.value());
        canonical.push(group_order_sort_value(*order));
    }
}

fn push_len(canonical: &mut Vec<u8>, len: usize) {
    canonical.extend_from_slice(&(len as u32).to_le_bytes());
}

fn push_string(canonical: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    push_len(canonical, bytes.len());
    canonical.extend_from_slice(bytes);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AuthoritativeTick(u64);

impl AuthoritativeTick {
    const ZERO: Self = Self(0);

    const fn value(self) -> u64 {
        self.0
    }
}

fn calculate_next_tick(
    started_tick: AuthoritativeTick,
) -> Result<AuthoritativeTick, TickExecutionError> {
    started_tick
        .value()
        .checked_add(1)
        .map(AuthoritativeTick)
        .ok_or(TickExecutionError::TickLimitReached {
            current_tick: started_tick.value(),
        })
}

fn record_trace(
    trace: &mut Option<&mut ExecutionTrace>,
    tick: AuthoritativeTick,
    kind: TraceRecordKind,
) {
    if let Some(trace) = trace.as_deref_mut() {
        trace.record(tick, kind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_protocol::{
        CommandEnvelope, CommandPayload, CommandSequence, GameplayEvent, GroupConfig, GroupId,
        GroupOrder, IssueGroupOrder, MatchConfig, MatchId, ParticipantConfig, ParticipantId,
        RobotId, Seed, TargetTick, TeamConfig, TeamId, TickRateHz,
    };

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

    fn match_config(seed: u64) -> MatchConfig {
        MatchConfig::new(
            id_match("match-001"),
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(seed),
            vec![],
            vec![],
            vec![],
        )
        .expect("empty roster is explicit and valid")
    }

    fn populated_match_config() -> MatchConfig {
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

    fn reordered_populated_match_config() -> MatchConfig {
        MatchConfig::new(
            id_match("match-001"),
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(123456),
            vec![
                TeamConfig::new(id_team("team-red")),
                TeamConfig::new(id_team("team-blue")),
            ],
            vec![
                ParticipantConfig::new(id_participant("participant-red-1"), id_team("team-red")),
                ParticipantConfig::new(id_participant("participant-blue-1"), id_team("team-blue")),
            ],
            vec![
                GroupConfig::new(
                    id_group("group-red-alpha"),
                    id_participant("participant-red-1"),
                    vec![id_robot("robot-red-002"), id_robot("robot-red-001")],
                ),
                GroupConfig::new(
                    id_group("group-blue-alpha"),
                    id_participant("participant-blue-1"),
                    vec![id_robot("robot-blue-002"), id_robot("robot-blue-001")],
                ),
            ],
        )
        .expect("valid reordered roster should pass validation")
    }

    fn group_order_command(
        sequence: u64,
        target_tick: u64,
        participant_id: ParticipantId,
        group_id: GroupId,
    ) -> CommandEnvelope {
        CommandEnvelope::new(
            CommandSequence::new(sequence),
            TargetTick::new(target_tick),
            participant_id,
            CommandPayload::IssueGroupOrder(IssueGroupOrder::new(
                group_id,
                GroupOrder::HoldPosition,
            )),
        )
    }

    fn command_sequences(results: &[CommandResult]) -> Vec<u64> {
        results
            .iter()
            .map(|result| result.sequence().value())
            .collect()
    }

    fn group_order_event_groups(events: &[GameplayEvent]) -> Vec<&str> {
        events
            .iter()
            .map(|event| match event {
                GameplayEvent::GroupOrderAssigned(event) => event.group_id().value(),
            })
            .collect()
    }

    #[test]
    fn new_simulation_starts_at_tick_zero() {
        let simulation = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");

        assert_eq!(simulation.current_tick(), 0);
    }

    #[test]
    fn valid_roster_with_teams_participants_and_groups_creates_a_simulation() {
        let simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");

        assert_eq!(simulation.current_tick(), 0);
        assert_eq!(simulation.match_config().match_id().value(), "match-001");
    }

    #[test]
    fn participant_may_issue_hold_position_to_a_group_they_control() {
        let simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let command = group_order_command(
            1,
            0,
            id_participant("participant-blue-1"),
            id_group("group-blue-alpha"),
        );

        let validation = simulation.validate_command_for_intent(&command);

        assert_eq!(validation.result.status(), CommandResultStatus::Accepted);
        assert_eq!(
            validation.intent,
            Some(CommandIntent::IssueGroupOrder(GroupOrderIntent {
                group_id: id_group("group-blue-alpha"),
                order: GroupOrder::HoldPosition,
            }))
        );
    }

    #[test]
    fn command_for_unknown_participant_is_rejected() {
        let simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let command = group_order_command(
            1,
            0,
            id_participant("participant-green-1"),
            id_group("group-blue-alpha"),
        );

        let validation = simulation.validate_command_for_intent(&command);

        assert_eq!(
            validation.result.status(),
            CommandResultStatus::Rejected {
                reason: CommandRejectionReason::UnknownParticipant,
            }
        );
        assert_eq!(validation.intent, None);
    }

    #[test]
    fn command_for_unknown_group_is_rejected() {
        let simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let command = group_order_command(
            1,
            0,
            id_participant("participant-blue-1"),
            id_group("group-blue-missing"),
        );

        let validation = simulation.validate_command_for_intent(&command);

        assert_eq!(
            validation.result.status(),
            CommandResultStatus::Rejected {
                reason: CommandRejectionReason::UnknownGroup,
            }
        );
        assert_eq!(validation.intent, None);
    }

    #[test]
    fn participant_commanding_another_participants_group_is_rejected() {
        let simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let command = group_order_command(
            1,
            0,
            id_participant("participant-blue-1"),
            id_group("group-red-alpha"),
        );

        let validation = simulation.validate_command_for_intent(&command);

        assert_eq!(
            validation.result.status(),
            CommandResultStatus::Rejected {
                reason: CommandRejectionReason::GroupNotControlledByParticipant,
            }
        );
        assert_eq!(validation.intent, None);
    }

    #[test]
    fn command_assigned_to_a_different_tick_is_rejected() {
        let simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let command = group_order_command(
            1,
            1,
            id_participant("participant-blue-1"),
            id_group("group-blue-alpha"),
        );

        let validation = simulation.validate_command_for_intent(&command);

        assert_eq!(
            validation.result.status(),
            CommandResultStatus::Rejected {
                reason: CommandRejectionReason::WrongTargetTick,
            }
        );
        assert_eq!(validation.intent, None);
    }

    #[test]
    fn accepted_hold_position_becomes_the_groups_active_order() {
        let mut simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let command = group_order_command(
            1,
            0,
            id_participant("participant-blue-1"),
            id_group("group-blue-alpha"),
        );

        let result = simulation
            .execute_tick_with_commands(&[command])
            .expect("command tick should complete");

        assert_eq!(
            result.command_results()[0].status(),
            CommandResultStatus::Accepted
        );
        assert_eq!(
            simulation.active_group_order(&id_group("group-blue-alpha")),
            Some(GroupOrder::HoldPosition)
        );
    }

    #[test]
    fn accepted_command_produces_one_authoritative_group_order_event() {
        let mut simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let command = group_order_command(
            1,
            0,
            id_participant("participant-blue-1"),
            id_group("group-blue-alpha"),
        );

        let result = simulation
            .execute_tick_with_commands(&[command])
            .expect("command tick should complete");

        assert_eq!(result.gameplay_events().len(), 1);
        assert_eq!(
            result.gameplay_events()[0],
            GameplayEvent::GroupOrderAssigned(sim_protocol::GroupOrderAssignedEvent::new(
                0,
                sim_protocol::EventOrdinal::new(0),
                id_group("group-blue-alpha"),
                id_participant("participant-blue-1"),
                GroupOrder::HoldPosition,
            ))
        );
    }

    #[test]
    fn rejected_command_does_not_change_group_state_or_produce_event() {
        let mut simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let command = group_order_command(
            1,
            0,
            id_participant("participant-blue-1"),
            id_group("group-red-alpha"),
        );

        let result = simulation
            .execute_tick_with_commands(&[command])
            .expect("command tick should complete");

        assert_eq!(
            result.command_results()[0].status(),
            CommandResultStatus::Rejected {
                reason: CommandRejectionReason::GroupNotControlledByParticipant,
            }
        );
        assert_eq!(
            simulation.active_group_order(&id_group("group-red-alpha")),
            None
        );
        assert!(result.gameplay_events().is_empty());
    }

    #[test]
    fn commands_are_processed_in_sequence_order_not_input_order() {
        let mut simulation = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let later = group_order_command(
            2,
            0,
            id_participant("participant-red-1"),
            id_group("group-red-alpha"),
        );
        let earlier = group_order_command(
            1,
            0,
            id_participant("participant-blue-1"),
            id_group("group-blue-alpha"),
        );

        let result = simulation
            .execute_tick_with_commands(&[later, earlier])
            .expect("command tick should complete");

        assert_eq!(command_sequences(result.command_results()), vec![1, 2]);
        assert_eq!(
            group_order_event_groups(result.gameplay_events()),
            vec!["group-blue-alpha", "group-red-alpha"]
        );
    }

    #[test]
    fn equivalent_command_sets_in_different_input_order_produce_identical_outputs_and_hashes() {
        let first_commands = vec![
            group_order_command(
                2,
                0,
                id_participant("participant-red-1"),
                id_group("group-red-alpha"),
            ),
            group_order_command(
                1,
                0,
                id_participant("participant-blue-1"),
                id_group("group-blue-alpha"),
            ),
        ];
        let second_commands = vec![first_commands[1].clone(), first_commands[0].clone()];
        let mut first = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let mut second = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");

        let first_result = first
            .execute_tick_with_commands(&first_commands)
            .expect("first command tick should complete");
        let second_result = second
            .execute_tick_with_commands(&second_commands)
            .expect("second command tick should complete");

        assert_eq!(
            first_result.command_results(),
            second_result.command_results()
        );
        assert_eq!(
            first_result.gameplay_events(),
            second_result.gameplay_events()
        );
        assert_eq!(first_result.state_hash(), second_result.state_hash());
        assert_eq!(first.state_hash(), second.state_hash());
    }

    #[test]
    fn duplicate_command_sequences_are_rejected_deterministically() {
        let duplicate_blue = group_order_command(
            1,
            0,
            id_participant("participant-blue-1"),
            id_group("group-blue-alpha"),
        );
        let duplicate_red = group_order_command(
            1,
            0,
            id_participant("participant-red-1"),
            id_group("group-red-alpha"),
        );
        let mut first = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let mut second = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");

        let first_result = first
            .execute_tick_with_commands(&[duplicate_blue.clone(), duplicate_red.clone()])
            .expect("first duplicate command tick should complete");
        let second_result = second
            .execute_tick_with_commands(&[duplicate_red, duplicate_blue])
            .expect("second duplicate command tick should complete");

        assert_eq!(
            first_result.command_results(),
            second_result.command_results()
        );
        assert!(first_result.command_results().iter().all(|result| {
            result.status()
                == CommandResultStatus::Rejected {
                    reason: CommandRejectionReason::DuplicateCommandSequence,
                }
        }));
        assert!(first_result.gameplay_events().is_empty());
        assert_eq!(
            first.active_group_order(&id_group("group-blue-alpha")),
            None
        );
        assert_eq!(first.active_group_order(&id_group("group-red-alpha")), None);
    }

    #[test]
    fn enabling_trace_does_not_change_command_outputs_events_state_or_hash() {
        let command = group_order_command(
            1,
            0,
            id_participant("participant-blue-1"),
            id_group("group-blue-alpha"),
        );
        let mut without_trace = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let mut with_trace = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let mut trace = ExecutionTrace::new();

        let without_trace_result = without_trace
            .execute_tick_with_commands(std::slice::from_ref(&command))
            .expect("tick without trace should complete");
        let with_trace_result = with_trace
            .execute_tick_with_commands_and_trace(&[command], Some(&mut trace))
            .expect("tick with trace should complete");

        assert_eq!(
            without_trace_result.command_results(),
            with_trace_result.command_results()
        );
        assert_eq!(
            without_trace_result.gameplay_events(),
            with_trace_result.gameplay_events()
        );
        assert_eq!(
            without_trace.active_group_order(&id_group("group-blue-alpha")),
            with_trace.active_group_order(&id_group("group-blue-alpha"))
        );
        assert_eq!(
            without_trace_result.state_hash(),
            with_trace_result.state_hash()
        );
        assert_eq!(without_trace.state_hash(), with_trace.state_hash());
    }

    #[test]
    fn one_successful_tick_completes_tick_one() {
        let mut simulation = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");

        let result = simulation
            .execute_tick()
            .expect("empty authoritative tick should complete");

        assert_eq!(result.started_tick(), 0);
        assert_eq!(result.completed_tick(), 1);
        assert_eq!(simulation.current_tick(), 1);
    }

    #[test]
    fn three_successful_ticks_complete_tick_three() {
        let mut simulation = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");

        let first = simulation.execute_tick().expect("tick 1 should complete");
        let second = simulation.execute_tick().expect("tick 2 should complete");
        let third = simulation.execute_tick().expect("tick 3 should complete");

        assert_eq!(
            [
                first.completed_tick(),
                second.completed_tick(),
                third.completed_tick()
            ],
            [1, 2, 3]
        );
        assert_eq!(simulation.current_tick(), 3);
    }

    #[test]
    fn identical_configurations_follow_the_same_tick_sequence() {
        let mut first = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");
        let mut second = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");

        let first_ticks = [
            first
                .execute_tick()
                .expect("first simulation tick 1 should complete")
                .completed_tick(),
            first
                .execute_tick()
                .expect("first simulation tick 2 should complete")
                .completed_tick(),
            first
                .execute_tick()
                .expect("first simulation tick 3 should complete")
                .completed_tick(),
        ];
        let second_ticks = [
            second
                .execute_tick()
                .expect("second simulation tick 1 should complete")
                .completed_tick(),
            second
                .execute_tick()
                .expect("second simulation tick 2 should complete")
                .completed_tick(),
            second
                .execute_tick()
                .expect("second simulation tick 3 should complete")
                .completed_tick(),
        ];

        assert_eq!(first_ticks, second_ticks);
    }

    #[test]
    fn tick_execution_requires_no_time_input() {
        let mut simulation = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");

        let result = simulation.execute_tick();

        assert!(result.is_ok());
    }

    #[test]
    fn identical_runs_produce_the_same_hash_after_every_tick() {
        let mut first = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");
        let mut second = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");

        assert_eq!(first.state_hash(), second.state_hash());

        for _ in 0..3 {
            let first_result = first.execute_tick().expect("first tick should complete");
            let second_result = second.execute_tick().expect("second tick should complete");

            assert_eq!(first_result.state_hash(), second_result.state_hash());
            assert_eq!(first.state_hash(), second.state_hash());
        }
    }

    #[test]
    fn different_seeds_produce_different_initial_hashes() {
        let first = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");
        let second = MatchSimulation::new(match_config(654321))
            .expect("validated configuration should create a simulation");

        assert_ne!(first.state_hash(), second.state_hash());
    }

    #[test]
    fn different_tick_counts_produce_different_final_hashes() {
        let mut one_tick = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");
        let mut two_ticks = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");

        one_tick.execute_tick().expect("tick 1 should complete");
        two_ticks.execute_tick().expect("tick 1 should complete");
        two_ticks.execute_tick().expect("tick 2 should complete");

        assert_ne!(one_tick.state_hash(), two_ticks.state_hash());
    }

    #[test]
    fn equivalent_rosters_in_different_input_order_produce_the_same_initial_hash() {
        let first = MatchSimulation::new(populated_match_config())
            .expect("validated roster should create a simulation");
        let second = MatchSimulation::new(reordered_populated_match_config())
            .expect("validated reordered roster should create a simulation");

        assert_eq!(first.state_hash(), second.state_hash());
    }

    #[test]
    fn enabling_trace_does_not_change_the_hash() {
        let mut without_trace = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");
        let mut with_trace = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");
        let mut trace = ExecutionTrace::new();

        let without_trace_result = without_trace
            .execute_tick()
            .expect("tick without trace should complete");
        let with_trace_result = with_trace
            .execute_tick_with_trace(Some(&mut trace))
            .expect("tick with trace should complete");

        assert_eq!(
            without_trace_result.state_hash(),
            with_trace_result.state_hash()
        );
        assert_eq!(without_trace.state_hash(), with_trace.state_hash());
    }

    #[test]
    fn trace_reflects_the_actual_execution_order() {
        let mut simulation = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");
        let mut trace = ExecutionTrace::new();

        simulation
            .execute_tick_with_trace(Some(&mut trace))
            .expect("traced tick should complete");

        let trace_kinds = trace
            .records()
            .iter()
            .map(TraceRecord::kind)
            .collect::<Vec<_>>();
        let trace_ticks = trace
            .records()
            .iter()
            .map(TraceRecord::tick)
            .collect::<Vec<_>>();

        assert_eq!(
            trace_kinds,
            vec![
                TraceRecordKind::TickStarted,
                TraceRecordKind::CommandsSelected,
                TraceRecordKind::CommandsNormalized,
                TraceRecordKind::GameplayEventsFinalized,
                TraceRecordKind::TickTransitionCalculated,
                TraceRecordKind::TickTransitionApplied,
                TraceRecordKind::StateHashCalculated,
                TraceRecordKind::TickCompleted,
            ]
        );
        assert_eq!(trace_ticks, vec![0, 0, 0, 0, 0, 1, 1, 1]);
    }
}
