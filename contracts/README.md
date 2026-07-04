# Contracts <!-- omit in toc -->

- [Core Contracts](#core-contracts)
- [Example Contracts](#example-contracts)
- [Utility Contracts](#utility-contracts)

---

## Core Contracts

- [**core-token-vesting**](./core-token-vesting/README.md)
- [**core-token-vesting-v2**](./core-token-vesting-v2/README.md)
- [**core-shifter**](./core-shifter/README.md): Simple contract to execute peg shift
  and depth shift admin calls in x/perp module. This contract is meant to be used
  to run a bot.
- [**core-controller**](./core-controller): Admin calls for things like creating
  perp markets or changing oracle parameters.
- [**core-compounder**](./core-compounder): Simple contract to allow third parties
  to stake funds without being able to withdraw/unstake them.

## Broker Contracts

Account abstraction smart contracts where the smart contract acts as or enables a
broker to act on the contract owner's behalf with some gated functionality.

- [**broker-bank**](./broker-bank/README.md): Account abstration to enable funds
  to be held and sent to a whitelisted set of accounts (`TO_ADDRS`). Bank
  transfers can only be called by "operators", and the funds can only be
  withdrawn by the contract owner.
- [**broker-staking**](./broker-staking/README.md): Account abstraction to enable
  certain staking transaction messages to be called by a subset of "operators".
  Although operators can stake the funds, only the contract owner can withdraw or
  unstake them.


## Example Contracts

- [**nibi-stargate**](./nibi-stargate/README.md): Example smart contract that showcases how to use the Nibiru standard (nibiru-std) library to execute `CosmosMsg::Stargate` transactions for the token factory module.

## Utility Contracts

- **lockup**: Smart contract that enables users to lock or bond tokens for arbitrary durations. This contract can be used as a building block in combination with a contract like `incentives` to implement liquidity mining incentives or other yield mechanisms.

- **incentives**: Smart contract for funding lockups based with tokens.
