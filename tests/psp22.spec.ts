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
    const INITIAL_SUPPLY = BigInt(100 * 10 ** DECIMALS);
    const MAX_SUPPLY = BigInt(1000 * 10 ** DECIMALS);
    const NAME = "PepeToken";
    const SYMBOL = "PEPE";
    const SALE_PRICE = BigInt(1 * 10 ** DECIMALS) / BigInt(10 ** DECIMALS);
    const START_AT = 0;
    const END_AT = 1707868800000;

    async function setup(): Promise<void> {

        api = await ApiPromise.create({ provider: wsProvider });
        deployer = keyring.addFromUri("//Alice");
        bob = keyring.addFromUri("//Bob");
        projectAccount = keyring.addFromUri("//Charlie");
        baseTokenFactory = new BaseToken_factory(api, deployer);
        contract = new BaseToken(
            (await baseTokenFactory.new(
                INITIAL_SUPPLY.toString(),
                Array.from((new TextEncoder()).encode(NAME)),
                Array.from((new TextEncoder()).encode(SYMBOL)),
                DECIMALS,
                true,
                true,
                true,
                true,
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

        const transferAmount = BigInt(1 * 10 ** DECIMALS)

        await contract.withSigner(deployer).tx.transfer(bob.address, transferAmount.toString(), [], null);

        const balanceAfter = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        expect(balanceBefore - balanceAfter).to.equal(transferAmount);
    })

    it("Mint works", async () => {
        await setup();

        const balanceBefore = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        const mintAmount = BigInt(1 * 10 ** DECIMALS)

        await contract.withSigner(deployer).tx.mintTo(deployer.address, mintAmount.toString());

        const balanceAfter = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        expect(balanceAfter - balanceBefore).to.equal(mintAmount);
    })

    it("Burn works", async () => {
        await setup();

        const balanceBefore = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        const burnAmount = BigInt(1 * 10 ** DECIMALS)

        await contract.withSigner(deployer).tx.burn(burnAmount.toString());

        const balanceAfter = BigInt((await contract.query.balanceOf(deployer.address, null)).value.unwrap().toString());

        expect(balanceBefore - balanceAfter).to.equal(burnAmount);
    })

    it("Get works", async () => {
        await setup();
        const is_pausable = (await contract.query.getIsPausable()).value.ok.ok;
        const is_mintable = (await contract.query.getIsMintable()).value.ok.ok;
        const is_burnable = (await contract.query.getIsBurnable()).value.ok.ok;
        const is_sale = (await contract.query.getIsSale()).value.ok.ok;

        expect(is_pausable).to.be.equal(true);
        expect(is_mintable).to.be.equal(true);
        expect(is_burnable).to.be.equal(true);
        expect(is_sale).to.be.equal(true);
    })

    it("Create sale", async () => {
        await setup();

        await contract.withSigner(deployer).tx.setSaleOptions(SALE_PRICE.toString(), MAX_SUPPLY.toString(), START_AT, END_AT);

        const sale_price = (await contract.query.getSaleRate()).value.ok.ok.toString();
        const max_supply = (await contract.query.getMaxSupply()).value.ok.ok.toString();
        const start_at = (await contract.query.getStartAt()).value.ok.ok;
        const end_at = (await contract.query.getEndAt()).value.ok.ok;

        expect(sale_price).to.equal(SALE_PRICE.toString());
        expect(max_supply).to.equal(MAX_SUPPLY.toString());
        expect(start_at).to.equal(START_AT);
        expect(end_at).to.equal(END_AT);
    })

    it("Buy works", async () => {
        await setup();

        await contract.withSigner(deployer).tx.setSaleOptions(SALE_PRICE.toString(), MAX_SUPPLY.toString(), START_AT, END_AT);

        const balanceBefore = BigInt((await contract.query.balanceOf(bob.address, null)).value.unwrap().toString());

        const amount = BigInt(0.5 * 10 ** DECIMALS);

        const result = await contract.withSigner(bob).query.buy();

        await contract
            .withSigner(bob)
            .tx.buy({ value: (SALE_PRICE * amount).toString() });

        const balanceAfter = BigInt((await contract.query.balanceOf(bob.address, null)).value.unwrap().toString());
        expect(balanceAfter - balanceBefore).to.equal(amount);

    })
})