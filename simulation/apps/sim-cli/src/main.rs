#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use sim_core::{ExecutionTrace, MatchCreationError, MatchSimulation, TickExecutionError};
use sim_protocol::{
    CommandEnvelope, CommandRejectionReason, CommandResult, CommandResultStatus, GameplayEvent,
    GroupOrder, HeadlessScenario, IdentifierKind, IdentifierValidationError,
    MatchConfigValidationError, ScenarioInputError, ScenarioValidationError,
    parse_headless_scenario_json,
};

fn main() -> ExitCode {
    match run_from_args(env::args().skip(1)) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run_from_args(args: impl IntoIterator<Item = String>) -> Result<String, CliError> {
    let args = args.into_iter().collect::<Vec<_>>();
    match args.as_slice() {
        [command, scenario_path] if command == "run" => run_scenario(Path::new(scenario_path)),
        _ => Err(CliError::Usage),
    }
}

fn run_scenario(path: &Path) -> Result<String, CliError> {
    let source = fs::read_to_string(path).map_err(|source| CliError::ReadScenario {
        path: path.to_path_buf(),
        source,
    })?;
    let scenario = parse_headless_scenario_json(&source).map_err(CliError::ScenarioInput)?;
    let result = execute_scenario(scenario)?;

    serde_json::to_string_pretty(&result).map_err(CliError::SerializeResult)
}

fn execute_scenario(scenario: HeadlessScenario) -> Result<RunOutput, CliError> {
    let mut simulation =
        MatchSimulation::new(scenario.match_config().clone()).map_err(CliError::CreateMatch)?;
    let initial_tick = simulation.current_tick();
    let mut completed_ticks = 0;
    let mut trace = ExecutionTrace::new();
    let mut command_results = Vec::new();
    let mut gameplay_events = Vec::new();

    for _ in 0..scenario.run_ticks() {
        let commands = commands_for_tick(scenario.commands(), simulation.current_tick());
        let tick_result = if scenario.trace_enabled() {
            simulation
                .execute_tick_with_commands_and_trace(&commands, Some(&mut trace))
                .map_err(CliError::ExecuteTick)?
        } else {
            simulation
                .execute_tick_with_commands(&commands)
                .map_err(CliError::ExecuteTick)?
        };

        command_results.extend(
            tick_result
                .command_results()
                .iter()
                .map(command_result_output),
        );
        gameplay_events.extend(
            tick_result
                .gameplay_events()
                .iter()
                .map(gameplay_event_output),
        );
        completed_ticks += 1;
    }

    let trace = if scenario.trace_enabled() {
        Some(trace_output_records(&trace))
    } else {
        None
    };

    Ok(RunOutput {
        schema_version: scenario.schema_version().value(),
        match_id: scenario.match_config().match_id().value().to_owned(),
        initial_tick,
        completed_ticks,
        final_tick: simulation.current_tick(),
        state_hash: state_hash_hex(simulation.state_hash().bytes()),
        command_results,
        gameplay_events,
        trace,
    })
}

fn commands_for_tick(commands: &[CommandEnvelope], tick: u64) -> Vec<CommandEnvelope> {
    commands
        .iter()
        .filter(|command| command.target_tick().value() == tick)
        .cloned()
        .collect()
}

#[derive(Debug)]
enum CliError {
    Usage,
    ReadScenario {
        path: PathBuf,
        source: std::io::Error,
    },
    ScenarioInput(ScenarioInputError),
    CreateMatch(MatchCreationError),
    ExecuteTick(TickExecutionError),
    SerializeResult(serde_json::Error),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage => write!(formatter, "usage: sim-cli run <scenario-path>"),
            Self::ReadScenario { path, source } => {
                write!(
                    formatter,
                    "failed to read scenario '{}': {source}",
                    path.display()
                )
            }
            Self::ScenarioInput(error) => {
                write!(
                    formatter,
                    "invalid scenario: {}",
                    format_scenario_input_error(error)
                )
            }
            Self::CreateMatch(error) => write!(formatter, "failed to create match: {error:?}"),
            Self::ExecuteTick(error) => write!(formatter, "failed to execute tick: {error:?}"),
            Self::SerializeResult(error) => {
                write!(formatter, "failed to serialize result: {error}")
            }
        }
    }
}

fn format_scenario_input_error(error: &ScenarioInputError) -> String {
    match error {
        ScenarioInputError::DataShape { message } => {
            format!("invalid data shape: {message}")
        }
        ScenarioInputError::Validation(error) => format_scenario_validation_error(error),
    }
}

fn format_scenario_validation_error(error: &ScenarioValidationError) -> String {
    match error {
        ScenarioValidationError::UnsupportedSchemaVersion { found, supported } => {
            format!(
                "unsupported schema version {}, supported version is {}",
                found.value(),
                supported.value()
            )
        }
        ScenarioValidationError::InvalidTickRate { found_hz } => {
            format!("invalid tick rate {found_hz}; expected a positive value")
        }
        ScenarioValidationError::InvalidRunTicks { found } => {
            format!("invalid run ticks {found}; expected a positive value")
        }
        ScenarioValidationError::InvalidIdentifier { error } => format_identifier_error(error),
        ScenarioValidationError::InvalidMatchConfig { error } => format_match_config_error(error),
    }
}

fn format_identifier_error(error: &IdentifierValidationError) -> String {
    match error {
        IdentifierValidationError::Empty { kind } => {
            format!(
                "invalid {} identifier; expected a non-empty value",
                identifier_kind_name(*kind)
            )
        }
    }
}

fn identifier_kind_name(kind: IdentifierKind) -> &'static str {
    match kind {
        IdentifierKind::Match => "match",
        IdentifierKind::Team => "team",
        IdentifierKind::Participant => "participant",
        IdentifierKind::Group => "group",
        IdentifierKind::Robot => "robot",
    }
}

fn format_match_config_error(error: &MatchConfigValidationError) -> String {
    match error {
        MatchConfigValidationError::DuplicateTeamId { team_id } => {
            format!("duplicate team identifier '{}'", team_id.value())
        }
        MatchConfigValidationError::DuplicateParticipantId { participant_id } => {
            format!(
                "duplicate participant identifier '{}'",
                participant_id.value()
            )
        }
        MatchConfigValidationError::UnknownParticipantTeam {
            participant_id,
            team_id,
        } => format!(
            "participant '{}' references unknown team '{}'",
            participant_id.value(),
            team_id.value()
        ),
        MatchConfigValidationError::DuplicateGroupId { group_id } => {
            format!("duplicate group identifier '{}'", group_id.value())
        }
        MatchConfigValidationError::UnknownGroupController {
            group_id,
            controller_participant_id,
        } => format!(
            "group '{}' references unknown controlling participant '{}'",
            group_id.value(),
            controller_participant_id.value()
        ),
        MatchConfigValidationError::DuplicateRobotId { robot_id } => {
            format!("duplicate robot identifier '{}'", robot_id.value())
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunOutput {
    schema_version: u16,
    match_id: String,
    initial_tick: u64,
    completed_ticks: u32,
    final_tick: u64,
    state_hash: String,
    command_results: Vec<CommandResultOutput>,
    gameplay_events: Vec<GameplayEventOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<Vec<TraceRecordOutput>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandResultOutput {
    sequence: u64,
    target_tick: u64,
    participant_id: String,
    status: CommandResultStatusOutput,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandResultStatusOutput {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GameplayEventOutput {
    kind: &'static str,
    tick: u64,
    ordinal: u32,
    group_id: String,
    participant_id: String,
    order: GroupOrderOutput,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupOrderOutput {
    kind: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TraceRecordOutput {
    tick: u64,
    kind: &'static str,
}

fn command_result_output(result: &CommandResult) -> CommandResultOutput {
    CommandResultOutput {
        sequence: result.sequence().value(),
        target_tick: result.target_tick().value(),
        participant_id: result.participant_id().value().to_owned(),
        status: command_result_status_output(result.status()),
    }
}

fn command_result_status_output(status: CommandResultStatus) -> CommandResultStatusOutput {
    match status {
        CommandResultStatus::Accepted => CommandResultStatusOutput {
            kind: "Accepted",
            reason: None,
        },
        CommandResultStatus::Rejected { reason } => CommandResultStatusOutput {
            kind: "Rejected",
            reason: Some(command_rejection_reason_name(reason)),
        },
    }
}

fn command_rejection_reason_name(reason: CommandRejectionReason) -> &'static str {
    match reason {
        CommandRejectionReason::WrongTargetTick => "WrongTargetTick",
        CommandRejectionReason::UnknownParticipant => "UnknownParticipant",
        CommandRejectionReason::UnknownGroup => "UnknownGroup",
        CommandRejectionReason::GroupNotControlledByParticipant => {
            "GroupNotControlledByParticipant"
        }
        CommandRejectionReason::DuplicateCommandSequence => "DuplicateCommandSequence",
    }
}

fn gameplay_event_output(event: &GameplayEvent) -> GameplayEventOutput {
    match event {
        GameplayEvent::GroupOrderAssigned(event) => GameplayEventOutput {
            kind: "GroupOrderAssigned",
            tick: event.tick(),
            ordinal: event.ordinal().value(),
            group_id: event.group_id().value().to_owned(),
            participant_id: event.participant_id().value().to_owned(),
            order: group_order_output(event.order()),
        },
    }
}

fn group_order_output(order: GroupOrder) -> GroupOrderOutput {
    match order {
        GroupOrder::HoldPosition => GroupOrderOutput {
            kind: "HoldPosition",
        },
    }
}

fn trace_output_records(trace: &ExecutionTrace) -> Vec<TraceRecordOutput> {
    trace
        .records()
        .iter()
        .map(|record| TraceRecordOutput {
            tick: record.tick(),
            kind: trace_kind_name(record.kind()),
        })
        .collect()
}

fn trace_kind_name(kind: sim_core::TraceRecordKind) -> &'static str {
    match kind {
        sim_core::TraceRecordKind::TickStarted => "TickStarted",
        sim_core::TraceRecordKind::CommandsSelected => "CommandsSelected",
        sim_core::TraceRecordKind::CommandsNormalized => "CommandsNormalized",
        sim_core::TraceRecordKind::CommandValidationCompleted => "CommandValidationCompleted",
        sim_core::TraceRecordKind::IntentProduced => "IntentProduced",
        sim_core::TraceRecordKind::IntentApplied => "IntentApplied",
        sim_core::TraceRecordKind::GameplayEventsFinalized => "GameplayEventsFinalized",
        sim_core::TraceRecordKind::TickTransitionCalculated => "TickTransitionCalculated",
        sim_core::TraceRecordKind::TickTransitionApplied => "TickTransitionApplied",
        sim_core::TraceRecordKind::StateHashCalculated => "StateHashCalculated",
        sim_core::TraceRecordKind::TickCompleted => "TickCompleted",
    }
}

fn state_hash_hex(bytes: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}
