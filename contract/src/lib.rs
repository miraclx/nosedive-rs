use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{collections::LookupMap, AccountId};
use near_sdk::{env, near_bindgen, require};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Votes {
    given: u64,
    received: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserState {
    rating: f32,
    votes: Votes,
}

impl Default for UserState {
    fn default() -> Self {
        Self {
            rating: 2.0, // you're a 2 simply for existing
            votes: Votes {
                given: 0,
                received: 1,
            },
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NoseDive {
    records: LookupMap<AccountId, UserState>,
}

impl Default for NoseDive {
    fn default() -> Self {
        Self {
            records: LookupMap::new(":r".as_bytes()),
        }
    }
}

fn validate_rating(rating: f32) -> bool {
    let fract = rating.fract();
    (rating > 0.0 && rating <= 5.0) && (fract == 0.0 || fract == 0.5)
}

#[near_bindgen]
impl NoseDive {
    pub fn vote_for(&mut self, account_id: AccountId, rating: f32) {
        require!(
            validate_rating(rating),
            "enter a valid rating between 0.5-5.0 (steps by 0.5)"
        );
        println!("{} {}", account_id, env::signer_account_id());
        assert!(
            account_id != env::signer_account_id(),
            "you can't rate yourself {} {}",
            account_id,
            env::signer_account_id()
        );
        let my_account_id = env::signer_account_id();
        let mut my_state = self.records.get(&my_account_id).unwrap_or_default();
        let mut them_state = self.records.get(&account_id).unwrap_or_default();
        them_state.rating = ((them_state.rating * them_state.votes.received as f32)
            + (rating * my_state.rating) / 5.0)
            / {
                my_state.votes.given += 1;
                them_state.votes.received += 1;
                them_state.votes.received as f32
            };
        self.records.insert(&my_account_id, &my_state);
        self.records.insert(&account_id, &them_state);
    }

    pub fn get_rating(&self, account_id: AccountId) -> Option<f32> {
        self.records
            .get(&account_id)
            .map(|user_state| user_state.rating)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn set_then_get_rating() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = NoseDive::default();
        contract.vote_for("alice_near".parse().unwrap(), 1.0);
        assert_eq!(
            Some(1.2),
            contract.get_rating("alice_near".parse().unwrap())
        );
    }

    #[test]
    fn validate_5_star_as_fract() {
        for rating in [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0] {
            assert!(
                validate_rating(rating),
                "valid rating specification marked invalid: {:.1}",
                rating
            );
        }
        for rating in [
            f32::NEG_INFINITY,
            f32::NAN,
            -2.5,
            -0.1,
            0.0,
            0.1,
            0.2,
            0.3,
            0.4,
            5.5,
            10.0,
            f32::INFINITY,
        ] {
            assert!(
                !validate_rating(rating),
                "invalid rating specification marked valid: {:.1}",
                rating
            );
        }
    }
}
