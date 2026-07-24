#![forbid(unsafe_code)]

use sim_protocol::MatchConfig;

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
}

impl TickResult {
    pub const fn started_tick(self) -> u64 {
        self.started_tick
    }

    pub const fn completed_tick(self) -> u64 {
        self.completed_tick
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

    pub fn execute_tick(&mut self) -> Result<TickResult, TickExecutionError> {
        let started_tick = self.verify_next_tick_may_begin()?;
        let completed_tick = calculate_next_tick(started_tick)?;

        self.apply_tick_transition(completed_tick);

        Ok(TickResult {
            started_tick: started_tick.value(),
            completed_tick: completed_tick.value(),
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
}
