use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

use crate::msg::RegistryMode;

pub const STATE: Item<State> = Item::new("state");

#[cw_serde]
pub struct State {
    pub count: u64,
    pub registry_mode: RegistryMode,
    pub target_addr: Option<String>,
    pub valid_payload_increment: u64,
    pub last_sudo: Option<String>,
}
