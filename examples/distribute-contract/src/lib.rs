use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, require, AccountId, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct DistributeContract {
    owner_id: AccountId,
    friends: Vec<AccountId>,
}

impl Default for DistributeContract {
    fn default() -> Self {
        Self {
            owner_id: "near".parse().unwrap(),
            friends: Vec::new(),
        }
    }
}

#[near_bindgen]
impl DistributeContract {
    #[init]
    pub fn init(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            friends: Vec::new(),
        }
    }

    #[payable]
    pub fn deposit(&mut self) {
        require!(
            env::signer_account_id() == self.owner_id,
            "only owner is allowed to make deposits"
        );
    }

    pub fn add_friend(&mut self, friend_id: AccountId) {
        require!(
            env::signer_account_id() == self.owner_id,
            "only owner is allowed to add a friend"
        );
        self.friends.push(friend_id);
    }

    pub fn pay_friends(&mut self) -> Promise {
        require!(
            env::signer_account_id() == self.owner_id,
            "only owner is allowed to pay out friends"
        );
        let per_friend_amount = env::account_balance() / self.friends.len() as u128;
        let mut result_promise: Option<Promise> = None;
        for friend_id in &self.friends {
            let friend_promise = Promise::new(friend_id.clone()).transfer(per_friend_amount);
            if let Some(promise) = result_promise {
                result_promise = Some(promise.and(friend_promise));
            } else {
                result_promise = Some(friend_promise);
            }
        }
        result_promise.unwrap()
    }
}
