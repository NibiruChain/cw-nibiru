use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};

use crate::{
    msg::{
        ExecuteMsg, HookDispatch, InstantiateMsg, QueryMsg, RegistryMode,
        SudoMsg,
    },
    state::{State, STATE},
};

type ContractError = anyhow::Error;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    STATE.save(
        deps.storage,
        &State {
            count: msg.count,
            registry_mode: msg.registry_mode,
            target_addr: msg.target_addr,
            valid_payload_increment: msg.valid_payload_increment.unwrap_or(1),
            last_sudo: None,
        },
    )?;

    Ok(Response::default().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetRegistryMode { mode } => {
            STATE.update(
                deps.storage,
                |mut state| -> Result<_, ContractError> {
                    state.registry_mode = mode;
                    Ok(state)
                },
            )?;
            Ok(Response::default().add_attribute("method", "set_registry_mode"))
        }
        ExecuteMsg::SetTargetAddr { target_addr } => {
            STATE.update(
                deps.storage,
                |mut state| -> Result<_, ContractError> {
                    state.target_addr = target_addr;
                    Ok(state)
                },
            )?;
            Ok(Response::default().add_attribute("method", "set_target_addr"))
        }
        ExecuteMsg::SetCount { count } => {
            STATE.update(
                deps.storage,
                |mut state| -> Result<_, ContractError> {
                    state.count = count;
                    Ok(state)
                },
            )?;
            Ok(Response::default().add_attribute("method", "set_count"))
        }
        ExecuteMsg::SetValidPayloadIncrement { by } => {
            STATE.update(
                deps.storage,
                |mut state| -> Result<_, ContractError> {
                    state.valid_payload_increment = by;
                    Ok(state)
                },
            )?;
            Ok(Response::default()
                .add_attribute("method", "set_valid_payload_increment"))
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::BeginBlockPlan {} | QueryMsg::EndBlockPlan {} => {
            to_json_binary(&query_plan(deps, env)?).map_err(ContractError::from)
        }
        QueryMsg::State {} => to_json_binary(&STATE.load(deps.storage)?)
            .map_err(ContractError::from),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(
    deps: DepsMut,
    _env: Env,
    msg: SudoMsg,
) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::Increment { by } => {
            STATE.update(
                deps.storage,
                |mut state| -> Result<_, ContractError> {
                    state.count += by;
                    state.last_sudo = Some("increment".to_string());
                    Ok(state)
                },
            )?;
            Ok(Response::default()
                .add_attribute("method", "sudo_increment")
                .add_attribute("by", by.to_string()))
        }
        SudoMsg::Set { count } => {
            STATE.update(
                deps.storage,
                |mut state| -> Result<_, ContractError> {
                    state.count = count;
                    state.last_sudo = Some("set".to_string());
                    Ok(state)
                },
            )?;
            Ok(Response::default()
                .add_attribute("method", "sudo_set")
                .add_attribute("count", count.to_string()))
        }
        SudoMsg::FailBeforeWrite {} => {
            anyhow::bail!("fixture sudo failure before write")
        }
        SudoMsg::FailAfterWrite { by } => {
            STATE.update(
                deps.storage,
                |mut state| -> Result<_, ContractError> {
                    state.count += by;
                    state.last_sudo = Some("fail_after_write".to_string());
                    Ok(state)
                },
            )?;
            anyhow::bail!("fixture sudo failure after write")
        }
    }
}

fn query_plan(deps: Deps, env: Env) -> Result<Vec<HookDispatch>, ContractError> {
    let state = STATE.load(deps.storage)?;
    let target_addr = state
        .target_addr
        .clone()
        .unwrap_or_else(|| env.contract.address.to_string());
    let valid_payload = serde_json::to_value(SudoMsg::Increment {
        by: state.valid_payload_increment,
    })?;

    match state.registry_mode {
        RegistryMode::Empty {} => Ok(vec![]),
        RegistryMode::SingleValid {} => Ok(vec![HookDispatch {
            contract_addr: target_addr,
            msg: valid_payload,
        }]),
        RegistryMode::MixedValidAndInvalid {} => Ok(vec![
            HookDispatch {
                contract_addr: target_addr.clone(),
                msg: valid_payload.clone(),
            },
            HookDispatch {
                contract_addr: "not-a-wasm-contract-address".to_string(),
                msg: valid_payload,
            },
            HookDispatch {
                contract_addr: target_addr,
                msg: serde_json::json!("not-a-sudo-object"),
            },
        ]),
        RegistryMode::QueryError {} => {
            anyhow::bail!("fixture registry query error")
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    use crate::{
        contract::{execute, instantiate, query, sudo},
        msg::{
            ExecuteMsg, HookDispatch, InstantiateMsg, QueryMsg, RegistryMode,
            SudoMsg,
        },
        state::State,
    };

    const SENDER: &str = "sender";
    const TARGET: &str = "target_contract";

    fn setup(
        registry_mode: RegistryMode,
    ) -> anyhow::Result<(
        cosmwasm_std::OwnedDeps<
            cosmwasm_std::testing::MockStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        cosmwasm_std::Env,
    )> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info(SENDER, &[]),
            InstantiateMsg {
                count: 0,
                registry_mode,
                target_addr: Some(TARGET.to_string()),
                valid_payload_increment: Some(7),
            },
        )?;
        Ok((deps, env))
    }

    fn query_state(
        deps: cosmwasm_std::Deps,
        env: cosmwasm_std::Env,
    ) -> anyhow::Result<State> {
        Ok(from_json(query(deps, env, QueryMsg::State {})?)?)
    }

    #[test]
    fn registry_mode_empty_returns_no_dispatches() -> anyhow::Result<()> {
        let (deps, env) = setup(RegistryMode::Empty {})?;

        let dispatches: Vec<HookDispatch> =
            from_json(query(deps.as_ref(), env, QueryMsg::BeginBlockPlan {})?)?;

        assert!(dispatches.is_empty());
        Ok(())
    }

    #[test]
    fn registry_mode_single_valid_returns_target_dispatch() -> anyhow::Result<()>
    {
        let (deps, env) = setup(RegistryMode::SingleValid {})?;

        let dispatches: Vec<HookDispatch> =
            from_json(query(deps.as_ref(), env, QueryMsg::EndBlockPlan {})?)?;

        assert_eq!(dispatches.len(), 1);
        assert_eq!(dispatches[0].contract_addr, TARGET);
        let sudo_msg: SudoMsg =
            serde_json::from_value(dispatches[0].msg.clone())?;
        assert_eq!(sudo_msg, SudoMsg::Increment { by: 7 });
        Ok(())
    }

    #[test]
    fn registry_mode_mixed_returns_valid_and_invalid_dispatches(
    ) -> anyhow::Result<()> {
        let (deps, env) = setup(RegistryMode::MixedValidAndInvalid {})?;

        let dispatches: Vec<HookDispatch> =
            from_json(query(deps.as_ref(), env, QueryMsg::BeginBlockPlan {})?)?;

        assert_eq!(dispatches.len(), 3);
        assert_eq!(dispatches[0].contract_addr, TARGET);
        assert_eq!(dispatches[1].contract_addr, "not-a-wasm-contract-address");
        assert!(serde_json::from_value::<SudoMsg>(dispatches[2].msg.clone())
            .is_err());
        Ok(())
    }

    #[test]
    fn registry_mode_query_error_fails_query() -> anyhow::Result<()> {
        let (deps, env) = setup(RegistryMode::QueryError {})?;

        let err = query(deps.as_ref(), env, QueryMsg::BeginBlockPlan {})
            .expect_err("query should fail");

        assert!(err.to_string().contains("fixture registry query error"));
        Ok(())
    }

    #[test]
    fn sudo_increment_mutates_counter() -> anyhow::Result<()> {
        let (mut deps, env) = setup(RegistryMode::Empty {})?;

        let res =
            sudo(deps.as_mut(), env.clone(), SudoMsg::Increment { by: 5 })?;

        assert_eq!(res.attributes[0].value, "sudo_increment");
        let state = query_state(deps.as_ref(), env)?;
        assert_eq!(state.count, 5);
        assert_eq!(state.last_sudo, Some("increment".to_string()));
        Ok(())
    }

    #[test]
    fn sudo_fail_before_write_leaves_counter_unchanged() -> anyhow::Result<()> {
        let (mut deps, env) = setup(RegistryMode::Empty {})?;

        let err = sudo(deps.as_mut(), env.clone(), SudoMsg::FailBeforeWrite {})
            .expect_err("sudo should fail");

        assert!(err.to_string().contains("failure before write"));
        let state = query_state(deps.as_ref(), env)?;
        assert_eq!(state.count, 0);
        assert_eq!(state.last_sudo, None);
        Ok(())
    }

    #[test]
    fn sudo_fail_after_write_mutates_before_returning_error(
    ) -> anyhow::Result<()> {
        let (mut deps, env) = setup(RegistryMode::Empty {})?;

        let err = sudo(
            deps.as_mut(),
            env.clone(),
            SudoMsg::FailAfterWrite { by: 9 },
        )
        .expect_err("sudo should fail");

        assert!(err.to_string().contains("failure after write"));
        let state = query_state(deps.as_ref(), env)?;
        assert_eq!(state.count, 9);
        assert_eq!(state.last_sudo, Some("fail_after_write".to_string()));
        Ok(())
    }

    #[test]
    fn execute_updates_registry_settings() -> anyhow::Result<()> {
        let (mut deps, env) = setup(RegistryMode::Empty {})?;

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info(SENDER, &[]),
            ExecuteMsg::SetRegistryMode {
                mode: RegistryMode::SingleValid {},
            },
        )?;
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info(SENDER, &[]),
            ExecuteMsg::SetTargetAddr {
                target_addr: Some("other_target".to_string()),
            },
        )?;

        let dispatches: Vec<HookDispatch> =
            from_json(query(deps.as_ref(), env, QueryMsg::BeginBlockPlan {})?)?;
        assert_eq!(dispatches[0].contract_addr, "other_target");
        Ok(())
    }

    #[test]
    fn instantiate_defaults_target_to_contract_address() -> anyhow::Result<()> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info(SENDER, &[]),
            InstantiateMsg {
                count: 0,
                registry_mode: RegistryMode::SingleValid {},
                target_addr: None,
                valid_payload_increment: None,
            },
        )?;

        let dispatches: Vec<HookDispatch> = from_json(query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::BeginBlockPlan {},
        )?)?;
        assert_eq!(
            dispatches[0].contract_addr,
            Addr::unchecked(env.contract.address).to_string()
        );
        Ok(())
    }

    #[test]
    fn golden_json_payloads_are_stable() -> anyhow::Result<()> {
        let begin_block = serde_json::to_string(&QueryMsg::BeginBlockPlan {})?;
        let end_block = serde_json::to_string(&QueryMsg::EndBlockPlan {})?;
        let increment = serde_json::to_string(&SudoMsg::Increment { by: 7 })?;
        let set = serde_json::to_string(&SudoMsg::Set { count: 42 })?;
        let fail_before = serde_json::to_string(&SudoMsg::FailBeforeWrite {})?;
        let fail_after =
            serde_json::to_string(&SudoMsg::FailAfterWrite { by: 9 })?;
        let dispatch = serde_json::to_string(&HookDispatch {
            contract_addr: TARGET.to_string(),
            msg: serde_json::to_value(SudoMsg::Increment { by: 7 })?,
        })?;

        println!("begin_block_query={begin_block}");
        println!("end_block_query={end_block}");
        println!("sudo_increment={increment}");
        println!("sudo_set={set}");
        println!("sudo_fail_before_write={fail_before}");
        println!("sudo_fail_after_write={fail_after}");
        println!("single_dispatch={dispatch}");

        assert_eq!(begin_block, r#"{"begin_block_plan":{}}"#);
        assert_eq!(end_block, r#"{"end_block_plan":{}}"#);
        assert_eq!(increment, r#"{"increment":{"by":7}}"#);
        assert_eq!(set, r#"{"set":{"count":42}}"#);
        assert_eq!(fail_before, r#"{"fail_before_write":{}}"#);
        assert_eq!(fail_after, r#"{"fail_after_write":{"by":9}}"#);
        assert_eq!(
            dispatch,
            r#"{"contract_addr":"target_contract","msg":{"increment":{"by":7}}}"#
        );
        Ok(())
    }
}
