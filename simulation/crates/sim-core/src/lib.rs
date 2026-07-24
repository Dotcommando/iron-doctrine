#![forbid(unsafe_code)]

use sim_protocol::MatchConfig;

const AUTHORITATIVE_STATE_VERSION: u16 = 1;
const STATE_HASH_ALGORITHM: &str = "BLAKE3-256";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchCreationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickExecutionError {
    TickLimitReached { current_tick: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickResult {
    started_tick: u64,
    completed_tick: u64,
    state_hash: StateHash,
}

impl TickResult {
    pub const fn started_tick(self) -> u64 {
        self.started_tick
    }

    pub const fn completed_tick(self) -> u64 {
        self.completed_tick
    }

    pub const fn state_hash(self) -> StateHash {
        self.state_hash
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchSimulation {
    state: AuthoritativeState,
}

impl MatchSimulation {
    pub const fn new(config: MatchConfig) -> Result<Self, MatchCreationError> {
        Ok(Self {
            state: AuthoritativeState::new(config),
        })
    }

    pub const fn current_tick(&self) -> u64 {
        self.state.current_tick.value()
    }

    pub const fn match_config(&self) -> MatchConfig {
        self.state.config
    }

    pub fn state_hash(&self) -> StateHash {
        self.state.hash()
    }

    pub fn execute_tick(&mut self) -> Result<TickResult, TickExecutionError> {
        self.execute_tick_with_trace(None)
    }

    pub fn execute_tick_with_trace(
        &mut self,
        trace: Option<&mut ExecutionTrace>,
    ) -> Result<TickResult, TickExecutionError> {
        let mut trace = trace;
        let started_tick = self.verify_next_tick_may_begin()?;
        record_trace(&mut trace, started_tick, TraceRecordKind::TickStarted);

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AuthoritativeState {
    config: MatchConfig,
    current_tick: AuthoritativeTick,
}

impl AuthoritativeState {
    const fn new(config: MatchConfig) -> Self {
        Self {
            config,
            current_tick: AuthoritativeTick::ZERO,
        }
    }

    fn hash(self) -> StateHash {
        let mut canonical = Vec::with_capacity(2 + 2 + 8 + 8);
        canonical.extend_from_slice(&AUTHORITATIVE_STATE_VERSION.to_le_bytes());
        canonical.extend_from_slice(&self.config.tick_rate_hz().to_le_bytes());
        canonical.extend_from_slice(&self.config.seed().to_le_bytes());
        canonical.extend_from_slice(&self.current_tick.value().to_le_bytes());

        StateHash(*blake3::hash(&canonical).as_bytes())
    }
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
    use sim_protocol::{MatchConfig, Seed, TickRateHz};

    fn match_config(seed: u64) -> MatchConfig {
        MatchConfig::new(
            TickRateHz::new(20).expect("test tick rate is valid"),
            Seed::new(seed),
        )
    }

    #[test]
    fn new_simulation_starts_at_tick_zero() {
        let simulation = MatchSimulation::new(match_config(123456))
            .expect("validated configuration should create a simulation");

        assert_eq!(simulation.current_tick(), 0);
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
                TraceRecordKind::TickTransitionCalculated,
                TraceRecordKind::TickTransitionApplied,
                TraceRecordKind::StateHashCalculated,
                TraceRecordKind::TickCompleted,
            ]
        );
        assert_eq!(trace_ticks, vec![0, 0, 1, 1, 1]);
    }
}
