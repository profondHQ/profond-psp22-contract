# Token project using PSP22

This is an example token project using ink! smart contract. The project is generated with Openbrush wizard for PSP22.

### 🏗️ How to use - Contracts

##### Requirements

```
cargo +nightly-2023-03-13 install --force --locked cargo-contract@2.2.1
```

##### Build and deploy

Build the contract with

```
yarn compile
```

Then deploy contract using https://contracts-ui.substrate.io/

##### 💫 Run integration test

First start your local node (https://github.com/paritytech/substrate-contracts-node). This repo is tested using version substrate-contracts-node 0.24.0-fbc28a7ad4b.

And then:

```sh
yarn
yarn compile
yarn compile:release
yarn test
```
