#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod base_token {
    use ink::{codegen::EmitEvent, env::DefaultEnvironment, EnvAccess};
    use openbrush::{
        contracts::pausable::*,
        contracts::psp22::{extensions::metadata::*, PSP22Error},
        traits::{Storage, String},
    };

    pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

    #[derive(Default, Debug)]
    #[openbrush::upgradeable_storage(STORAGE_KEY)]
    pub struct EnabledFeatures {
        is_pausable: bool,
        is_mintable: bool,
        is_burnable: bool,
        owner_addresss: Option<AccountId>,
        // Sale features
        is_sale: bool,
        sale_price: Option<Balance>,
        max_supply: Option<Balance>,
        start_at: Option<Timestamp>,
        end_at: Option<Timestamp>,
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
        #[storage_field]
        features: EnabledFeatures,
        #[storage_field]
        pause: pausable::Data,
    }

    #[ink(event)]
    pub struct SetSaleOptions {
        sale_price: Balance,
        max_supply: Balance,
        start_at: Timestamp,
        end_at: Timestamp,
    }

    #[ink(event)]
    pub struct TokenBought {
        #[ink(topic)]
        receiver_address: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    impl PSP22 for Contract {}
    impl PSP22Metadata for Contract {}
    impl Pausable for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            initial_supply: Balance,
            name: Option<String>,
            symbol: Option<String>,
            decimals: u8,
            is_pausable: bool,
            is_mintable: bool,
            is_burnable: bool,
            is_sale: bool,
        ) -> Self {
            let mut instance = Self::default();

            assert!(instance
                ._mint_to(Self::env().caller(), initial_supply)
                .is_ok());

            instance.metadata.name = name;
            instance.metadata.symbol = symbol;
            instance.metadata.decimals = decimals;

            instance.features.is_pausable = is_pausable;
            instance.features.is_mintable = is_mintable;
            instance.features.is_burnable = is_burnable;
            instance.features.is_sale = is_sale;

            if is_mintable || is_pausable {
                instance.features.owner_addresss = Some(Self::env().caller());
            }

            instance
        }

        // Get functions

        #[ink(message)]
        pub fn get_is_sale(&self) -> Result<bool, PSP22Error> {
            Ok(self.features.is_sale)
        }

        #[ink(message)]
        pub fn get_is_pausable(&self) -> Result<bool, PSP22Error> {
            Ok(self.features.is_pausable)
        }

        #[ink(message)]
        pub fn get_is_mintable(&self) -> Result<bool, PSP22Error> {
            Ok(self.features.is_mintable)
        }

        #[ink(message)]
        pub fn get_is_burnable(&self) -> Result<bool, PSP22Error> {
            Ok(self.features.is_burnable)
        }

        #[ink(message)]
        pub fn get_sale_price(&self) -> Result<Balance, PSP22Error> {
            Ok(self.features.sale_price.unwrap())
        }

        #[ink(message)]
        pub fn get_start_at(&self) -> Result<Timestamp, PSP22Error> {
            Ok(self.features.start_at.unwrap())
        }

        #[ink(message)]
        pub fn get_end_at(&self) -> Result<Timestamp, PSP22Error> {
            Ok(self.features.end_at.unwrap())
        }

        #[ink(message)]
        pub fn get_max_supply(&self) -> Result<Balance, PSP22Error> {
            Ok(self.features.max_supply.unwrap())
        }

        fn emit_set_sale_options_event(
            &self,
            sale_price: Balance,
            max_supply: Balance,
            start_at: Timestamp,
            end_at: Timestamp,
        ) {
            <EnvAccess<'_, DefaultEnvironment> as EmitEvent<Contract>>::emit_event::<SetSaleOptions>(
                self.env(),
                SetSaleOptions {
                    sale_price,
                    max_supply,
                    start_at,
                    end_at,
                },
            );
        }

        fn emit_token_bought_event(&self, receiver_address: AccountId, amount: Balance) {
            <EnvAccess<'_, DefaultEnvironment> as EmitEvent<Contract>>::emit_event::<TokenBought>(
                self.env(),
                TokenBought {
                    receiver_address,
                    amount,
                },
            );
        }

        #[ink(message)]
        pub fn set_sale_options(
            &mut self,
            sale_price: Balance,
            max_supply: Balance,
            start_at: Timestamp,
            end_at: Timestamp,
        ) -> Result<(), PSP22Error> {
            if self.env().caller() != self.features.owner_addresss.unwrap() {
                return Err(PSP22Error::Custom(String::from("Not minter")));
            }

            self.features.sale_price = Some(sale_price);
            self.features.max_supply = Some(max_supply);
            self.features.start_at = Some(start_at);
            self.features.end_at = Some(end_at);

            self.emit_set_sale_options_event(sale_price, max_supply, start_at, end_at);

            Ok(())
        }

        #[ink(message, payable)]
        pub fn buy(&mut self, amount: Balance) -> Result<Balance, PSP22Error> {
            let receiver_address = self.env().caller();
            let transferred_value = self.env().transferred_value();
            let current_timestamp = self.env().block_timestamp();

            if !self.features.is_sale || self.features.sale_price.is_none() {
                return Err(PSP22Error::Custom(String::from("Feature not enabled")));
            }

            if current_timestamp < self.features.start_at.unwrap()
                || current_timestamp > self.features.end_at.unwrap()
            {
                return Err(PSP22Error::Custom(String::from("Not on sale")));
            }

            if transferred_value < (self.features.sale_price.unwrap() * amount) {
                return Err(PSP22Error::Custom(String::from("Insufficient funds")));
            }

            let total_supply = self.psp22.total_supply();

            if total_supply + amount > self.features.max_supply.unwrap() {
                return Err(PSP22Error::Custom(String::from("Insufficient supply")));
            }

            self._mint_to(receiver_address, amount)?;

            self.env()
                .transfer(self.features.owner_addresss.unwrap(), transferred_value);

            self.emit_token_bought_event(receiver_address, amount);

            Ok(amount)
        }

        #[ink(message)]
        pub fn change_state(&mut self) -> Result<(), PSP22Error> {
            if !self.features.is_pausable {
                return Err(PSP22Error::Custom(String::from("Feature not enabled")));
            }

            if self.env().caller() != self.features.owner_addresss.unwrap() {
                return Err(PSP22Error::Custom(String::from("Not minter")));
            }

            self._switch_pause()
        }

        #[ink(message)]
        pub fn mint_to(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            if !self.features.is_mintable {
                return Err(PSP22Error::Custom(String::from("Feature not enabled")));
            }

            if self.env().caller() != self.features.owner_addresss.unwrap() {
                return Err(PSP22Error::Custom(String::from("Not minter")));
            }

            self._mint_to(account, amount)
        }

        #[ink(message)]
        pub fn burn(&mut self, amount: Balance) -> Result<(), PSP22Error> {
            if !self.features.is_burnable {
                return Err(PSP22Error::Custom(String::from("Feature not enabled")));
            }

            self._burn_from(self.env().caller(), amount)
        }
    }
}
