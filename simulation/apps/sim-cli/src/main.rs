#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use sim_core::{ExecutionTrace, MatchCreationError, MatchSimulation, TickExecutionError};
use sim_protocol::{
    HeadlessScenario, ScenarioInputError, ScenarioValidationError, parse_headless_scenario_json,
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
        MatchSimulation::new(scenario.match_config()).map_err(CliError::CreateMatch)?;
    let initial_tick = simulation.current_tick();
    let mut completed_ticks = 0;
    let mut trace = ExecutionTrace::new();

    for _ in 0..scenario.run_ticks() {
        if scenario.trace_enabled() {
            simulation
                .execute_tick_with_trace(Some(&mut trace))
                .map_err(CliError::ExecuteTick)?;
        } else {
            simulation.execute_tick().map_err(CliError::ExecuteTick)?;
        }
        completed_ticks += 1;
    }

    let trace = if scenario.trace_enabled() {
        Some(trace_output_records(&trace))
    } else {
        None
    };

    Ok(RunOutput {
        schema_version: scenario.schema_version().value(),
        initial_tick,
        completed_ticks,
        final_tick: simulation.current_tick(),
        state_hash: state_hash_hex(simulation.state_hash().bytes()),
        trace,
    })
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
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunOutput {
    schema_version: u16,
    initial_tick: u64,
    completed_ticks: u32,
    final_tick: u64,
    state_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<Vec<TraceRecordOutput>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TraceRecordOutput {
    tick: u64,
    kind: &'static str,
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
