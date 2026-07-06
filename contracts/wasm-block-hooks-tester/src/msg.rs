use cosmwasm_schema::{cw_serde, QueryResponses};
use serde_json::Value;

use crate::state::State;

/// RegistryMode selects the deterministic block-hook plan returned by the
/// fixture registry queries.
#[cw_serde]
pub enum RegistryMode {
    /// Empty returns no dispatch items.
    Empty {},
    /// SingleValid returns one valid dispatch item for the configured target.
    SingleValid {},
    /// MixedValidAndInvalid returns one valid dispatch item followed by invalid
    /// target and payload cases for host validation tests.
    MixedValidAndInvalid {},
    /// QueryError makes the registry query fail.
    QueryError {},
}

/// InstantiateMsg configures both the registry behavior and the counter state
/// used by the fixture target contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// Initial counter value stored by the fixture target contract.
    pub count: u64,
    /// Initial registry mode used by begin-block and end-block plan queries.
    pub registry_mode: RegistryMode,
    /// Optional target contract address for generated dispatch items. If unset,
    /// the fixture uses its own contract address.
    pub target_addr: Option<String>,
    /// Optional increment value used in valid generated sudo payloads.
    pub valid_payload_increment: Option<u64>,
}

/// ExecuteMsg updates fixture settings so tests can switch scenarios without
/// redeploying the contract.
#[cw_serde]
pub enum ExecuteMsg {
    /// SetRegistryMode changes the registry query behavior.
    SetRegistryMode { mode: RegistryMode },
    /// SetTargetAddr changes the target address used in valid dispatch items.
    SetTargetAddr { target_addr: Option<String> },
    /// SetCount overwrites the counter value used to inspect target writes.
    SetCount { count: u64 },
    /// SetValidPayloadIncrement changes the increment in generated sudo payloads.
    SetValidPayloadIncrement { by: u64 },
}

/// SudoMsg is the target-contract message schema the Go host passes into the
/// fixture sudo entry point.
#[cw_serde]
pub enum SudoMsg {
    /// Increment adds `by` to the counter and records a successful sudo call.
    Increment { by: u64 },
    /// Set overwrites the counter and records a successful sudo call.
    Set { count: u64 },
    /// FailBeforeWrite returns an error before mutating state.
    FailBeforeWrite {},
    /// FailAfterWrite mutates state and then returns an error, allowing host
    /// tests to prove rollback behavior.
    FailAfterWrite { by: u64 },
}

/// HookDispatch is one registry-selected target sudo call.
#[cw_serde]
pub struct HookDispatch {
    /// Bech32 address of the target Wasm contract.
    pub contract_addr: String,
    /// JSON sudo message payload to pass to the target contract.
    pub msg: Value,
}

/// QueryMsg exposes registry planning queries and state inspection.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// BeginBlockPlan returns dispatch items for the begin-block hook.
    #[returns(Vec<HookDispatch>)]
    BeginBlockPlan {},

    /// EndBlockPlan returns dispatch items for the end-block hook.
    #[returns(Vec<HookDispatch>)]
    EndBlockPlan {},

    /// State returns fixture state for tests.
    #[returns(State)]
    State {},
}
