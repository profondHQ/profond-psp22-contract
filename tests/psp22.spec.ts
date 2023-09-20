import { expect, use } from "chai";
import chaiAsPromised from "chai-as-promised";
import BaseToken_factory from "../types/constructors/base_token"
import BaseToken from "../types/contracts/base_token"

import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";

use(chaiAsPromised);

// Create a new instance of contract
const wsProvider = new WsProvider("ws://127.0.0.1:9944");
// Create a keyring instance
const keyring = new Keyring({ type: "sr25519" });

describe("PSP22 Testing", () => {
    let baseTokenFactory: BaseToken_factory;
    let api: ApiPromise;
    let deployer: KeyringPair;
    let bob: KeyringPair;
    let projectAccount: KeyringPair;
    let contract: BaseToken;

    const DECIMALS = 18;
    const TOTAL_SUPPLY = BigInt(1000000000 * 10 ** DECIMALS);
    const NAME = "PepeToken";
    const SYMBOL = "PEPE";

    async function setup(): Promise<void> {

        api = await ApiPromise.create({ provider: wsProvider });
        deployer = keyring.addFromUri("//Alice");
        bob = keyring.addFromUri("//Bob");
        projectAccount = keyring.addFromUri("//Charlie");
        baseTokenFactory = new BaseToken_factory(api, deployer);
        contract = new BaseToken(
            (await baseTokenFactory.new(
                TOTAL_SUPPLY.toString(),
                Array.from((new TextEncoder()).encode(NAME)),
                Array.from((new TextEncoder()).encode(SYMBOL)),
                DECIMALS,
                true,
                true,
                true
            )).address,
            deployer,
            api
        );
    }

    it("Metadata works", async () => {
        await setup();

        expect((await contract.query.tokenDecimals()).value.unwrap()).to.equal(DECIMALS);
        expect(Buffer.from((await contract.query.tokenName()).value.unwrap().toString().replace("0x", ""), 'hex').toString()).to.equal(NAME);
        expect(Buffer.from((await contract.query.tokenSymbol()).value.unwrap().toString().replace("0x", ""), 'hex').toString()).to.equal(SYMBOL);
    })

    it("Transfer works", async () => {
        await setup();

        const balanceBefore = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        const transferAmount = BigInt(1000000 * 10 ** DECIMALS)

        await contract.withSigner(deployer).tx.transfer(bob.address, transferAmount.toString(), [], null);

        const balanceAfter = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        expect(balanceBefore - balanceAfter).to.equal(transferAmount);
    })

    it("Mint works", async () => {
        await setup();

        const balanceBefore = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        const mintAmount = BigInt(1000000 * 10 ** DECIMALS)

        await contract.withSigner(deployer).tx.mintTo(deployer.address, mintAmount.toString());

        const balanceAfter = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        expect(balanceAfter - balanceBefore).to.equal(mintAmount);
    })

    it("Burn works", async () => {
        await setup();

        const balanceBefore = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        const burnAmount = BigInt(1000000 * 10 ** DECIMALS)

        await contract.withSigner(deployer).tx.burn(burnAmount.toString());

        const balanceAfter = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        expect(balanceBefore - balanceAfter).to.equal(burnAmount);
    })
})