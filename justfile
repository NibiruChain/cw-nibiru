workspaces := "./packages"
# workspaces := "./packages ./core"

set dotenv-load
# Displays available recipes by running `just -l`.
setup:
  #!/usr/bin/env bash
  just -l

install:
  # https://crates.io/crates/clippy
  rustup component add clippy
  # https://crates.io/crates/cargo-llvm-cov
  cargo install cargo-llvm-cov
  # https://crates.io/crates/cosmwasm-check
  cargo install cosmwasm-check

wasm-all:
  bash scripts/wasm-out.sh

# Move binding artifacts to teh local nibiru wasmbin
wasm-export:
  bash scripts/wasm-export.sh

# Check if a Wasm smart contract binary is ready for the blockchain
wasm-check:
  cosmwasm-check artifacts/*.wasm

# Compiles a single CW contract to wasm bytecode.
# wasm-single:
#   bash scripts/wasm-out.sh --single

# Runs rustfmt
fmt:
  cargo fmt --all

# Runs rustfmt without updating
fmt-check:
  cargo fmt --all -- --check

# Compiles Rust code
build:
  cargo build

build-update:
  cargo update
  cargo build

# Clean target files and temp files
clean:
  cargo clean

# Run linter + fix
clippy:
  cargo clippy --fix --allow-dirty --allow-staged

# Run linter + check only
clippy-check:
  cargo clippy

# Test a specific package or contract
test *pkg:
  #!/usr/bin/env bash
  set -e;
  if [ -z "{{pkg}}" ]; then
    just test-all
  else
    RUST_BACKGTRACE="1" cargo test --package "{{pkg}}"
  fi

# Test everything in the workspace.
test-all:
  cargo test

# Test everything and output coverage report.
test-coverage:
  cargo llvm-cov --lcov --output-path lcov.info \
    --ignore-filename-regex .*buf\/[^\/]+\.rs$

alias t := tidy

# Format, lint, and test
tidy:
  just fmt
  just clippy
  just wasm-check
  just test

# Format, lint, update dependencies, and test
tidy-update: build-update
  just tidy

gen-schema:
    #!/usr/bin/env bash
    initial_dir=$PWD
    for dir in contracts/*; do
        dir_name=$(basename $dir)
        echo "Generating schema for $dir_name..."

        # Change to the contract directory
        if cd $dir; then
            # Check if 'cargo schema' can be run successfully
            if cargo schema; then
                # Move back to the initial directory
                cd $initial_dir
                # Create target schema directory if it doesn't exist
                mkdir -p schema/$dir_name
                # Move the generated schema to the target directory
                if ! mv $dir/schema schema/$dir_name; then
                    echo "Failed to move schema directory for $dir_name."
                fi
            else
                cd $initial_dir
            fi
        else
            echo "Failed to change directory to $dir."
        fi
    done

# Generate schema for all contracts and generate TypeScript code
gen-ts:
    #!/usr/bin/env bash
    just gen-schema

    SCHEMA_DIR="./schema"
    TS_OUT_DIR="./dist"
    mkdir -p $TS_OUT_DIR
    for schema_path in $(find $SCHEMA_DIR -name schema -type d | grep -v "^./schema$"); do
        contract_name=$(basename $(dirname $schema_path))
        echo "Generating TypeScript for $contract_name..."
        cosmwasm-ts-codegen generate \
            --plugin client \
            --schema $schema_path \
            --out $TS_OUT_DIR/$contract_name \
            --name $contract_name \
            --no-bundle
    done
