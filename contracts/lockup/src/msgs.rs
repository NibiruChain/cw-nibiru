use cosmwasm_std::{Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Lock { blocks: u64 },

    InitiateUnlock { id: u64 },

    WithdrawFunds { id: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    LocksByDenomUnlockingAfter {
        denom: String,
        unlocking_after: u64,
    },
    LocksByDenomAndAddressUnlockingAfter {
        denom: String,
        unlocking_after: u64,
        address: Addr,
    },
    LocksByDenomBetween {
        denom: String,
        locked_before: u64,
        unlocking_after: u64,
    },
    LocksByDenomAndAddressBetween {
        denom: String,
        address: Addr,
        locked_before: u64,
        unlocking_after: u64,
    },
}
