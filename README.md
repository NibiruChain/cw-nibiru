# NibiruChain/cw-nibiru

Smart contract sandbox for Nibiru Chain.

```bash
⚡ NibiruChain/cw-nibiru
├── 📂 artifacts         # compiled .wasm smart contracts for cw-nibiru
├── 📂 contracts         # Smart contracts for Nibiru Chain
    └── 📂 nibi-stargate # Example contract using nibiru-std for CosmosMsg::Stargate
    └── 📂 incentives    # Generalized incentives over time for locked tokens
    └── 📂 lockup        # For locking and unlocking tokens like LP tokens
    └── 📂 pricefeed     # CosmWasm prototype of the (now deprecated) x/pricefeed module.
    └── 📂 core-cw3-flex-msig # CW3-flex-multisig with stargate enabled.
    └── 📂 core-shifter       # Calls peg shift and depth shift in x/perp.
    └── 📂 core-controller    # Calls other admin calls from Nibiru foundation.
    └── 📂 core-token-vesting # Token linear vesting contracts with optional cliffs.
    └── 📂 core-token-vesting-v2 # Improved version of core-token-vesting-v2.
├── 📂 nibiru-std      # Nibiru Chain standard library for smart contracts
    └── 📦 proto       # Types and traits for QueryRequest::Stargate and CosmosMsg::Stargate
         └──           #   Includes constructors for Cosmos, IBC, and Nibiru. 
    └── 📦 bindings    # For sending CosmosMsg::Custom msgs on Nibiru (soon deprecated).
├── 📂 packages        # Other Rust packages
    └── 📦 bash-rs     # Easily run bash from Rust. Used for writing testable and maintainable scripts.
    └── 📦 nibi-dev    # Dev tooling package for Nibiru. 
    └── 📦 nibiru-macro  # Implements procedural macros for the "nibiru-macro" package. 
├── Cargo.toml
├── Cargo.lock
└── README.md
```
## Install Cargo

- Install `cargo` with `curl`
```bash
curl https://sh.rustup.rs -sSf | sh
```
- Check `cargo` version
```bash
cargo version
```

## Hacking

Install `just` to run project-specific commands.

```bash
cargo install just
```

You can view the list of available development commands with `just -ls`.

Ref: [github.com/casey/just](https://github.com/casey/just)
