use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseResult};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct TknIndex {
    pub total_supply: Balance,
    pub balances: UnorderedMap<AccountId, Balance>,
    pub assets: UnorderedMap<String, Balance>,
    pub asset_prices: UnorderedMap<String, u128>,
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    Balances,
    Assets,
    AssetPrices,
}

impl Default for TknIndex {
    fn default() -> Self {
        Self {
            total_supply: 0,
            balances: UnorderedMap::new(StorageKey::Balances),
            assets: UnorderedMap::new(StorageKey::Assets),
            asset_prices: UnorderedMap::new(StorageKey::AssetPrices),
        }
    }
}

#[near_bindgen]
impl TknIndex {
    pub fn mint(&mut self, account_id: AccountId, amounts: Vec<(String, U128)>) {
        let mut total_value = 0;

        // Calculate the total value of deposited assets
        for (asset, amount) in &amounts {
            let price = self.get_asset_price(asset);
            total_value += price * amount.0;
            self.assets.insert(&asset, &(self.assets.get(&asset).unwrap_or(0) + amount.0));
        }

        // Mint index tokens proportional to the deposited value
        let new_tokens = total_value; // Assuming 1:1 value for simplicity
        self.balances.insert(&account_id, &(self.balances.get(&account_id).unwrap_or(0) + new_tokens));
        self.total_supply += new_tokens;
    }

    pub fn redeem(&mut self, account_id: AccountId, amount: U128) {
        let balance = self.balances.get(&account_id).unwrap_or(0);
        assert!(balance >= amount.0, "Insufficient balance");

        let share = amount.0 as f64 / self.total_supply as f64;

        // Calculate the share of each asset to be redeemed
        let mut assets_to_redeem = Vec::new();
        for (asset, asset_balance) in &self.assets {
            let transfer_amount = (asset_balance * share as u128) as U128;
            assets_to_redeem.push((asset.clone(), transfer_amount));
        }

        // Attempt to swap each asset and handle failures
        for (asset, amount) in assets_to_redeem {
            self.swap_and_transfer(asset, amount, account_id.clone());
        }

        self.balances.insert(&account_id, &(balance - amount.0));
        self.total_supply -= amount.0;
    }

    fn swap_and_transfer(&self, asset: String, amount: U128, account_id: AccountId) {
        // Implement the swap logic here
        // For example, interact with Ref Finance to swap assets

        // Placeholder logic for swap
        let swap_result = true; // Assume the swap is successful

        if swap_result {
            // Transfer the swapped asset to the user
            self.transfer_asset(account_id, asset, amount);
        } else {
            // Handle swap failure
            // You can retry the swap or provide options for the user to retry
            env::log_str("Swap failed, please try again.");
        }
    }

    fn transfer_asset(&self, account_id: AccountId, asset: String, amount: U128) {
        if asset.starts_with("near:") {
            // Transfer NEAR
            Promise::new(account_id).transfer(amount.0);
        } else {
            // Transfer fungible token
            ext_ft::ft_transfer(
                account_id,
                amount,
                None,
                &asset,
                1,
                env::prepaid_gas() - env::used_gas(),
            );
        }
    }

    pub fn get_asset_price(&self, asset: &String) -> u128 {
        self.asset_prices.get(asset).unwrap_or(1)
    }

    pub fn update_asset_price(&mut self, asset: String, price: u128) {
        self.asset_prices.insert(&asset, &price);
    }

    pub fn balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.balances.get(&account_id).unwrap_or(0))
    }
}

// #[cfg(not(target_arch = "wasm32"))]
// #[cfg(test)]
// mod tests {
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::testing_env;
//
//     use super::*;
//
//     // Allows for modifying the environment of the mocked blockchain
//     fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
//         let mut builder = VMContextBuilder::new();
//         builder
//             .current_account_id(accounts(0))
//             .signer_account_id(predecessor_account_id.clone())
//             .predecessor_account_id(predecessor_account_id);
//         builder
//     }
//
//     #[test]
//     fn set_get_message() {
//         let mut context = get_context(accounts(1));
//         // Initialize the mocked blockchain
//         testing_env!(context.build());
//
//         // Set the testing environment for the subsequent calls
//         testing_env!(context.predecessor_account_id(accounts(1)).build());
//
//         let mut contract = StatusMessage::default();
//         contract.set_status("hello".to_string());
//         assert_eq!(
//             "hello".to_string(),
//             contract.get_status(accounts(1)).unwrap()
//         );
//     }
//
//     #[test]
//     fn get_nonexistent_message() {
//         let contract = StatusMessage::default();
//         assert_eq!(None, contract.get_status("francis.near".parse().unwrap()));
//     }
// }
