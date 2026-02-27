# Solana-Pinocchio-Flashloan

## Setup a new Pinocchio project

```bash
cargo new program-name --lib --edition 2021
cd program-name
cargo add pinocchio pinocchio-system pinocchio-log pinocchio-pubkey shank
bun init
```

Add config in `Cargo.toml`

```toml
[lib]
crate-type = ["lib", "cdylib"]
```

### Make Program ID

```bash
solana-keygen new -o target/deploy/pinocchio_flashloan-keypair.json
solana address -k target/deploy/pinocchio_flashloan-keypair.json
```

Paste it into

- lib.rs: declare_id! macro
- web3jsSetup.ts: vaultProgAddr

### Build the program

```bash
cargo build-sbf
```

### Make IDL via Shank for new instruction layout

```bash
cargo install shank-cli
shank idl -o idl
```

The idl folder in our project now contains pinocchio_vault.json which is our program's IDL.

### Make a client via @solana/kit and Codama

Codama takes the Shank IDL and emits a TypeScript client. The generated code includes instruction builders, account types, and small conveniences that keep your client code focused on composing transactions.

```bash
  pnpm install
  pnpm dlx codama init
  pnpm dlx codama run js
```

You'll see a clients/js/src/gene` 1qw12qw3erdft6yhj7u8iokl./90You'll see a clients/js/src/generated/ folder in our project with the program types our client code uses to send transactions to our program.
