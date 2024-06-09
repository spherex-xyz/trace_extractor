# Hardhat Local Fork Transaction Traces Gathering

> **TL;DR**:
> 
> Use this repo to create new flows and patterns, after an **update to the protocol** (assuming said protocol is already deployed on chain with an engine)


This code is meant to extract transaction traces from a Hardhat local fork, so we can parse them into flows and then patterns.
The process includes querying the local node for the relevant transactions (using the JSON-RPC interface) and the re-running said transaction to get the traces (in our own fork of the local node)

It assumes that the **engine is deployed** in the local fork and all the contracts are wired into it.
It's main use case is for updates in the protocol - if the client update the protocol (functions or whole contracts) he can run the new flows locally with a turned off engine, then run this executable to extract the traces. Then he'll send the traces to us and we will create the relevant flows and patterns, and upload them to the dashboard or straight to the engine.



## Pre-requisites

- rust

To install:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup
```

## Usage

to run this program from the development environment, run

```sh
cargo run
```

you can run the program with `--help` to see the available options and arguments, and their default values.

```sh

cargo run -- --help
```
(you need to add `--` before the arguments to pass them to the program and not to the cargo itself)


## Normal Flow

usually, here is how the client would use this program:

1. run the hardhat local fork (`npx hardhat node`)
   - this is usually a fork of some public network
2. turn off the engine (by sending transaction to engine contract)
   -  this is sometimes combined with the next step
3. run the scripts that test the new/updated/changed functionality - these will send transactions to the local host
   - DO NOT turn off the local hardhat node this. we still need it.
4. run this program to extract the traces into a json file

The extracted traces should be sent to us, and we will create the relevant flows and patterns.



### Command-Line Arguments

- `--url`: The URL of the Hardhat local network (optional, defaults to `http://localhost:8545`)
- `--start_block`: The starting block number (optional, if not provided will take the fork block)
- `--output-path` - The path to the output file (optional, defaults to `traces.json`)

