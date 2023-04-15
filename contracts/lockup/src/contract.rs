use crate::error::ContractError;
use crate::events::{
    new_coins_locked_event, new_funds_withdrawn_event, new_unlock_initiation_event,
};
use crate::msgs::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{locks, Lock, LOCKS_ID, NOT_UNLOCKING_BLOCK_IDENTIFIER};
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Deps, DepsMut, Env, Event, MessageInfo, Order, Response, StdResult,
    Storage, entry_point
};
use cw_storage_plus::Bound;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    LOCKS_ID.save(deps.storage, &0).unwrap();

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Lock { blocks } => {
            return execute_lock(deps, env, info, blocks);
        }

        ExecuteMsg::InitiateUnlock { id } => {
            return execute_initiate_unlock(deps, env, info, id);
        }

        ExecuteMsg::WithdrawFunds { id } => {
            return execute_withdraw_funds(deps, env, info, id);
        }
    }
}

pub(crate) fn execute_withdraw_funds(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let locks = locks();

    // we update the lock to mark funds have been withdrawn
    let lock = locks.update(deps.storage, id, |lock| -> Result<_, ContractError> {
        let mut lock = lock.ok_or(ContractError::NotFound(id))?;

        if lock.funds_withdrawn {
            return Err(ContractError::FundsAlreadyWithdrawn(id));
        }

        if lock.end_block < env.block.height {
            return Err(ContractError::NotMatured(id));
        }

        lock.funds_withdrawn = true;

        Ok(lock)
    })?;

    Ok(Response::new()
        .add_event(new_funds_withdrawn_event(id, &lock.coin))
        .add_message(BankMsg::Send {
            to_address: lock.owner.to_string(),
            amount: vec![lock.coin],
        }))
}

pub(crate) fn execute_initiate_unlock(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let locks = locks();

    // initiate unlock
    let lock = locks.update(deps.storage, id, |lock| -> Result<_, ContractError> {
        let mut lock = lock.ok_or(ContractError::NotFound(id))?;
        if lock.end_block != NOT_UNLOCKING_BLOCK_IDENTIFIER {
            return Err(ContractError::AlreadyUnlocking(id));
        }
        lock.end_block = env.block.height + lock.duration_blocks;
        Ok(lock)
    })?;

    // emit unlock initiation event
    Ok(Response::new().add_event(new_unlock_initiation_event(id, &lock.coin, lock.end_block)))
}

pub(crate) fn execute_lock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    blocks: u64,
) -> Result<Response, ContractError> {
    if blocks == 0 {
        return Err(ContractError::InvalidLockDuration);
    }

    if info.funds.is_empty() {
        return Err(ContractError::InvalidCoins("no funds".to_string()));
    }

    // create a lock for each coin sent
    let locks = locks();
    let mut events: Vec<Event> = Vec::with_capacity(info.funds.len());
    for coin in info.funds {
        if coin.amount.is_zero() {
            return Err(ContractError::InvalidCoins(format!(
                "zero coins in denom: {}",
                coin.denom
            )));
        }

        let id = LOCKS_ID
            .update(deps.storage, |id| -> StdResult<_> { return Ok(id + 1) })
            .expect("must never fail");

        locks
            .save(
                deps.storage,
                id,
                &Lock {
                    id: id,
                    coin: coin.clone(),
                    owner: info.sender.clone(),
                    duration_blocks: blocks,
                    start_block: env.block.height,
                    end_block: NOT_UNLOCKING_BLOCK_IDENTIFIER,
                    funds_withdrawn: false,
                },
            )
            .expect("must never fail");

        events.push(new_coins_locked_event(id, &coin))
    }

    Ok(Response::new().add_events(events))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LocksByDenomAndAddressUnlockingAfter {
            denom,
            unlocking_after,
            address,
        } => Ok(to_binary(
            &locks()
                .idx
                .addr_denom_end
                .sub_prefix((address, denom))
                .range(
                    deps.storage,
                    Some(Bound::inclusive((unlocking_after, 0))),
                    None,
                    Order::Ascending,
                )
                .map(|lock| -> Lock { lock.unwrap().1 })
                .filter(|lock| -> bool {
                    if env.block.height + lock.duration_blocks <= unlocking_after {
                        return false;
                    }
                    true
                })
                .collect::<Vec<Lock>>(),
        )?),

        QueryMsg::LocksByDenomUnlockingAfter {
            denom,
            unlocking_after,
        } => Ok(to_binary(
            &locks()
                .idx
                .denom_end
                .sub_prefix(denom)
                .range(
                    deps.storage,
                    Some(Bound::inclusive((unlocking_after, 0))),
                    None,
                    Order::Ascending,
                )
                .map(|lock| -> Lock { lock.unwrap().1 })
                .filter(|lock| -> bool {
                    // TODO: make this more efficient by indexing this in state
                    if env.block.height + lock.duration_blocks <= unlocking_after {
                        return false;
                    }
                    true
                })
                .collect::<Vec<Lock>>(),
        )?),

        QueryMsg::LocksByDenomBetween {
            denom,
            locked_before,
            unlocking_after,
        } => Ok(to_binary(
            &locks()
                .idx
                .denom_start
                .sub_prefix(denom)
                .range(
                    deps.storage,
                    None,
                    Some(Bound::exclusive((locked_before, u64::MAX))),
                    Order::Ascending,
                )
                .map(|lock| -> Lock { lock.unwrap().1 })
                .filter(|lock| -> bool {
                    return env.block.height + lock.duration_blocks > unlocking_after;
                })
                .collect::<Vec<Lock>>(),
        )?),

        QueryMsg::LocksByDenomAndAddressBetween {
            denom,
            address,
            locked_before,
            unlocking_after,
        } => Ok(to_binary(
            &locks()
                .idx
                .addr_denom_start
                .sub_prefix((address, denom))
                .range(
                    deps.storage,
                    None,
                    Some(Bound::exclusive((locked_before, u64::MAX))),
                    Order::Ascending,
                )
                .map(|lock| -> Lock { lock.unwrap().1 })
                .filter(|lock| -> bool {
                    lock.duration_blocks + env.block.height > unlocking_after
                })
                .collect::<Vec<Lock>>(),
        )?),
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute_lock, instantiate, query};
    use crate::msgs::{InstantiateMsg, QueryMsg};
    use crate::state::{Lock};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    
    use cosmwasm_std::{from_binary, Addr, Coin, DepsMut, Env};

    struct TestLock {
        owner: Addr,
        duration: u64,
        coins: Vec<Coin>,
    }

    fn init(deps: DepsMut) {
        instantiate(deps, mock_env(), mock_info("none", &[]), InstantiateMsg {}).unwrap();
    }

    fn create_lock(deps: DepsMut, env: &Env, lock: &TestLock) {
        execute_lock(
            deps,
            env.clone(),
            mock_info(lock.owner.to_string().as_str(), lock.coins.as_slice()),
            lock.duration,
        )
        .unwrap();
    }

    #[test]
    fn queries() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        init(deps.as_mut());

        let lock_1 = TestLock {
            owner: Addr::unchecked("alice"),
            duration: 100,
            coins: vec![Coin::new(100, "ATOM"), Coin::new(300, "LUNA")],
        };

        let lock_2 = TestLock {
            owner: Addr::unchecked("bob"),
            duration: 50,
            coins: vec![Coin::new(200, "ATOM"), Coin::new(700, "OSMO")],
        };

        create_lock(deps.as_mut(), &env, &lock_1);
        create_lock(deps.as_mut(), &env, &lock_2);

        assert_eq!(
            vec![Coin::new(100, "ATOM"), Coin::new(200, "ATOM")],
            from_binary::<Vec<Lock>>(
                &query(
                    deps.as_ref(),
                    env.clone(),
                    QueryMsg::LocksByDenomUnlockingAfter {
                        denom: "ATOM".to_string(),
                        unlocking_after: 1_000_000
                    }
                )
                .unwrap()
            )
            .unwrap()
            .iter()
            .map(|lock| -> Coin { lock.coin.clone() })
            .collect::<Vec<Coin>>()
        );

        assert_eq!(
            vec![Coin::new(100, "ATOM")],
            from_binary::<Vec<Lock>>(
                &query(
                    deps.as_ref(),
                    env.clone(),
                    QueryMsg::LocksByDenomAndAddressUnlockingAfter {
                        denom: "ATOM".to_string(),
                        unlocking_after: 1_000_000,
                        address: lock_1.owner,
                    }
                )
                .unwrap()
            )
            .unwrap()
            .iter()
            .map(|lock| -> Coin { lock.coin.clone() })
            .collect::<Vec<Coin>>()
        )
    }

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
