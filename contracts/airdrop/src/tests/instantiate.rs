use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Uint128, Addr, coins, StdError};

use crate::contract::instantiate;
use crate::msg::InstantiateMsg;
use crate::state::{Campaign, CAMPAIGN};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let info = mock_info("sender", &coins(1000, ""));
    let env = mock_env();
    let msg = InstantiateMsg {
        owner: Addr::unchecked("sender"),
        campaign_id: "campaign_id".to_string(),
        campaign_name: "campaign_name".to_string(),
        campaign_description: "campaign_description".to_string(),
    };

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    let campaign = CAMPAIGN.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        campaign,
        Campaign {
            owner: Addr::unchecked("sender"),
            unallocated_amount: Uint128::new(1000),
            campaign_id: "campaign_id".to_string(),
            campaign_name: "campaign_name".to_string(),
            campaign_description: "campaign_description".to_string(),
        }
    );
}

#[test]
fn test_instantiate_with_no_funds() {
    let mut deps = mock_dependencies();
    let info = mock_info("sender", &[]);
    let env = mock_env();
    let msg = InstantiateMsg {
        owner: Addr::unchecked("sender"),
        campaign_id: "campaign_id".to_string(),
        campaign_name: "campaign_name".to_string(),
        campaign_description: "campaign_description".to_string(),
    };

    let resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(resp, Err(StdError::generic_err("Only one coin is allowed")));
}

#[test]
fn test_instantiate_with_invalid_denom() {
    let mut deps = mock_dependencies();
    let info = mock_info("sender", &coins(1000, "foo"));
    let env = mock_env();
    let msg = InstantiateMsg {
        owner: Addr::unchecked("sender"),
        campaign_id: "campaign_id".to_string(),
        campaign_name: "campaign_name".to_string(),
        campaign_description: "campaign_description".to_string(),
    };

    let resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(resp, Err(StdError::generic_err("Only native tokens are allowed")));
}