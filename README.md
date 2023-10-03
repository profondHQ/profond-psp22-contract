# Token project using PSP22

This is an example token project using ink! smart contract. The project is generated with Openbrush wizard for PSP22 with added PayableMinted trait.

### 🏗️ How to use - Contracts

##### Build and deploy

Build the contract with

```
yarn compile
```

Then deploy contract using https://contracts-ui.substrate.io/

##### 💫 Run integration test

First start your local node (https://github.com/paritytech/substrate-contracts-node). And then:

```sh
yarn
yarn compile
yarn compile:release
yarn test
```
