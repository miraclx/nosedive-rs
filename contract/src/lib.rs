use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::{collections::LookupMap, AccountId};
use near_sdk::{env, near_bindgen, require};

#[derive(Eq, Debug, PartialEq, Serialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Votes {
    given: u64,
    received: u64,
}

#[derive(Debug, PartialEq, Serialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UserState {
    rating: f32,
    #[serde(flatten)]
    votes: Votes,
}

impl Default for UserState {
    fn default() -> Self {
        Self {
            rating: 2.0, // you're a 2 ⭐️ simply for existing
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
    (rating >= 0.0 && rating <= 5.0) && rating.fract() % 0.5 == 0.0
}

#[near_bindgen]
impl NoseDive {
    pub fn register(&mut self) {
        let your_account_id = env::signer_account_id();
        if self.records.contains_key(&your_account_id) {
            env::panic_str(&format!(
                "this account has already been registered: [{}]",
                your_account_id
            ));
        }
        self.records.insert(&your_account_id, &UserState::default());
    }

    fn lookup(&self, account_id: &AccountId) -> UserState {
        let state = self.records.get(&account_id);
        match state {
            Some(state) => state,
            None => env::panic_str(&format!(
                "account does not exist on this service: [{}]",
                account_id,
            )),
        }
    }

    pub fn status(&self, account_id: AccountId) -> UserState {
        self.lookup(&account_id)
    }

    pub fn rate(&mut self, account_id: AccountId, rating: f32) {
        require!(
            validate_rating(rating),
            "enter a valid rating: multiples of .5 between 0 and 5"
        );
        let your_account_id = env::signer_account_id();
        let mut your_state = self.lookup(&your_account_id);
        let mut their_state = self.lookup(&account_id);
        require!(account_id != your_account_id, "you can't rate yourself");
        let total_ratings = their_state.rating * their_state.votes.received as f32;
        let this_rating = (rating + your_state.rating) / 2.0;
        their_state.votes.received += 1;
        your_state.votes.given += 1;
        their_state.rating = (total_ratings + this_rating) / their_state.votes.received as f32;
        self.records.insert(&your_account_id, &your_state);
        self.records.insert(&account_id, &their_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{
        test_utils::{
            test_env::{alice, bob},
            VMContextBuilder,
        },
        testing_env,
    };

    fn sys() -> AccountId {
        "nosedive_sys.near".parse().unwrap()
    }

    fn stage(account_id: AccountId) -> NoseDive {
        let context = VMContextBuilder::new()
            .current_account_id(sys())
            .signer_account_id(account_id.clone())
            .predecessor_account_id(account_id)
            .build();
        testing_env!(context);
        NoseDive::default()
    }

    fn status(account_id: AccountId) -> UserState {
        stage(sys()).status(account_id)
    }

    #[test]
    fn default() {
        stage(alice()).register();
        stage(bob()).register();
        // --
        assert_eq!(
            status(alice()),
            UserState {
                rating: 2.0,
                votes: Votes {
                    given: 0,
                    received: 1,
                }
            }
        );
        assert_eq!(
            status(bob()),
            UserState {
                rating: 2.0,
                votes: Votes {
                    given: 0,
                    received: 1,
                }
            }
        );
    }

    #[test]
    fn set_then_get() {
        stage(alice()).register();
        stage(bob()).register();
        // --
        for _ in 1..=10 {
            stage(bob()).rate(alice(), 4.5);
            stage(alice()).rate(bob(), 5.0);
        }
        // --
        assert_eq!(
            status(alice()),
            UserState {
                rating: 3.7977424,
                votes: Votes {
                    given: 10,
                    received: 11,
                }
            }
        );
        assert_eq!(
            status(bob()),
            UserState {
                rating: 4.006109,
                votes: Votes {
                    given: 10,
                    received: 11,
                }
            }
        );
    }

    #[test]
    fn validate_5_star_as_fract() {
        for rating in [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0] {
            assert!(
                validate_rating(rating),
                "valid rating specification marked invalid: {:.1}",
                rating
            );
        }
        for rating in [0.1, 0.2, 0.3, 0.4, 3.4, 5.5, 10.0] {
            assert!(
                !validate_rating(rating),
                "invalid rating specification marked valid: {:.1}",
                rating
            );
        }
    }
}
