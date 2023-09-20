#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod base_token {
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

    impl PSP22 for Contract {}
    impl PSP22Metadata for Contract {}
    impl Pausable for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            total_supply: Balance,
            name: Option<String>,
            symbol: Option<String>,
            decimals: u8,
            is_pausable: bool,
            is_mintable: bool,
            is_burnable: bool,
        ) -> Self {
            let mut instance = Self::default();

            assert!(instance
                ._mint_to(Self::env().caller(), total_supply)
                .is_ok());

            instance.metadata.name = name;
            instance.metadata.symbol = symbol;
            instance.metadata.decimals = decimals;

            instance.features.is_pausable = is_pausable;
            instance.features.is_mintable = is_mintable;
            instance.features.is_burnable = is_burnable;

            if is_mintable || is_pausable {
                instance.features.owner_addresss = Some(Self::env().caller());
            }

            instance
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
