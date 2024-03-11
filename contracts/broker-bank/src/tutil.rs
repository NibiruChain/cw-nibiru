//! testing.rs: Test helpers for the contract

use cosmwasm_std::{
    testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier,
        MockStorage,
    },
    Env, MessageInfo, OwnedDeps,
};

use crate::{contract::instantiate, msgs::InstantiateMsg};

pub const TEST_OWNER: &str = "owner";
pub const TEST_DENOM: &str = "testdenom";

pub fn setup_contract(
    to_addrs: Vec<String>,
    opers: Vec<String>,
) -> anyhow::Result<(
    OwnedDeps<MockStorage, MockApi, MockQuerier>,
    Env,
    MessageInfo,
)> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(TEST_OWNER, &[]);

    let msg = InstantiateMsg {
        owner: info.sender.to_string(),
        to_addrs: to_addrs.into_iter().collect(),
        opers: opers.into_iter().collect(),
    };
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg)?;
    assert_eq!(0, res.messages.len());
    Ok((deps, env, info))
}

pub fn mock_info_for_sender(sender: &str) -> MessageInfo {
    mock_info(sender, &[])
}

pub fn mock_env_height(height: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env
}