#![forbid(unsafe_code)]

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaVersion(u16);

impl SchemaVersion {
    pub const SUPPORTED: Self = Self(1);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchConfig {
    tick_rate_hz: TickRateHz,
    seed: Seed,
}

impl MatchConfig {
    pub const fn new(tick_rate_hz: TickRateHz, seed: Seed) -> Self {
        Self { tick_rate_hz, seed }
    }

    pub const fn tick_rate_hz(self) -> u16 {
        self.tick_rate_hz.value()
    }

    pub const fn seed(self) -> u64 {
        self.seed.value()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeadlessScenario {
    schema_version: SchemaVersion,
    match_config: MatchConfig,
    run_ticks: RunTicks,
    trace: bool,
}

impl HeadlessScenario {
    pub const fn schema_version(self) -> SchemaVersion {
        self.schema_version
    }

    pub const fn match_config(self) -> MatchConfig {
        self.match_config
    }

    pub const fn run_ticks(self) -> u32 {
        self.run_ticks.value()
    }

    pub const fn trace_enabled(self) -> bool {
        self.trace
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioInputError {
    DataShape { message: String },
    Validation(ScenarioValidationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawHeadlessScenario {
    schema_version: u16,
    #[serde(rename = "match")]
    match_config: RawMatchConfig,
    run_ticks: u32,
    trace: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawMatchConfig {
    tick_rate_hz: u16,
    seed: u64,
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

    let tick_rate_hz = TickRateHz::new(raw.match_config.tick_rate_hz)?;
    let run_ticks = RunTicks::new(raw.run_ticks)?;

    Ok(HeadlessScenario {
        schema_version,
        match_config: MatchConfig::new(tick_rate_hz, Seed::new(raw.match_config.seed)),
        run_ticks,
        trace: raw.trace,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_scenario_json(seed: u64) -> String {
        format!(
            r#"{{
                "schemaVersion": 1,
                "match": {{
                    "tickRateHz": 20,
                    "seed": {seed}
                }},
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
        assert_eq!(scenario.match_config().tick_rate_hz(), 20);
        assert_eq!(scenario.run_ticks(), 3);
        assert!(scenario.trace_enabled());
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let json =
            valid_scenario_json(123456).replace(r#""schemaVersion": 1"#, r#""schemaVersion": 2"#);

        let error = parse_headless_scenario_json(&json).expect_err("version 2 is unsupported");

        assert_eq!(
            error,
            ScenarioInputError::Validation(ScenarioValidationError::UnsupportedSchemaVersion {
                found: SchemaVersion::new(2),
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
}
