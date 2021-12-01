use std::fmt;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{de, Deserialize, Serialize};
use near_sdk::{
    collections::{LazyOption, LookupMap},
    AccountId, Timestamp,
};
use near_sdk::{env, near_bindgen, require};

#[derive(Eq, Debug, PartialEq, Serialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Votes {
    given: u64,
    received: u64,
}

#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct VoteInterval {
    secs: u64,
    msg: String,
}

#[derive(Debug, PartialEq, Serialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UserState {
    rating: f32,
    #[serde(flatten)]
    votes: Votes,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NoseDive {
    users: LookupMap<AccountId, UserState>,
    history: LookupMap<(AccountId, AccountId), Timestamp>,
    vote_interval: LazyOption<Option<VoteInterval>>,
}

impl Default for VoteInterval {
    fn default() -> Self {
        Self {
            secs: 5 * 60,
            msg: "you can't vote more than once in 5 minutes".to_string(),
        }
    }
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

impl Default for NoseDive {
    fn default() -> Self {
        let mut vote_interval = LazyOption::new(":i".as_bytes(), None);
        if let None = vote_interval.get() {
            vote_interval.set(&Some(VoteInterval::default()));
        }
        Self {
            users: LookupMap::new(":u".as_bytes()),
            history: LookupMap::new(":h".as_bytes()),
            vote_interval,
        }
    }
}

#[derive(Eq, Debug, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct RatingTimestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    they_rated_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    you_rated_at: Option<Timestamp>,
}

#[near_bindgen]
impl NoseDive {
    pub fn register(&mut self) {
        let your_account_id = env::signer_account_id();
        if self.users.contains_key(&your_account_id) {
            env::panic_str(&format!(
                "this account has already been registered: [{}]",
                your_account_id
            ));
        }
        self.users.insert(&your_account_id, &UserState::default());
    }

    fn lookup(&self, account_id: &AccountId) -> UserState {
        let state = self.users.get(account_id);
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

    pub fn rating_timestamps(&self, account_id: AccountId) -> RatingTimestamps {
        let your_account_id = env::signer_account_id();
        for id in [&your_account_id, &account_id] {
            if !self.users.contains_key(&id) {
                env::panic_str(&format!("account does not exist on this service: [{}]", id,));
            }
        }
        let you_them = (your_account_id, account_id);
        let you_rated_at = self.history.get(&you_them);
        let they_you = (you_them.1, you_them.0);
        let they_rated_at = self.history.get(&they_you);
        RatingTimestamps {
            you_rated_at,
            they_rated_at,
        }
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
        let (your_account_id, account_id) = {
            let now = env::block_timestamp();
            let rating_pair = (your_account_id, account_id);
            if let Some(Some(vote_interval)) = self.vote_interval.get() {
                if let Some(last_timestamp) = self.history.get(&rating_pair) {
                    require!(
                        now.checked_sub(last_timestamp)
                            .map_or(false, |delta| delta / 1_000_000_000 >= vote_interval.secs),
                        &vote_interval.msg
                    );
                }
            }
            self.history.insert(&rating_pair, &now);
            rating_pair
        };
        let total_ratings = their_state.rating * their_state.votes.received as f32;
        let this_rating = (rating + your_state.rating) / 2.0;
        their_state.votes.received += 1;
        your_state.votes.given += 1;
        their_state.rating = (total_ratings + this_rating) / their_state.votes.received as f32;
        self.users.insert(&your_account_id, &your_state);
        self.users.insert(&account_id, &their_state);
    }

    /// ## Format
    ///
    /// ```json
    /// {
    ///   "voting_interval": null | {
    ///     "secs": 5,
    ///     "msg": "wait 5 seconds"
    ///   }
    /// }
    /// ```
    pub fn patch_state(&mut self, patches: ContractPatch) {
        require!(
            env::current_account_id() == env::signer_account_id(),
            "only the account that deployed this contract is permitted to call this method"
        );
        for patch in patches.0 {
            match patch {
                PatchSpec::SetVotingInterval(interval) => {
                    self.vote_interval.set(&interval);
                }
            }
        }
    }
}

fn validate_rating(rating: f32) -> bool {
    (rating >= 0.0 && rating <= 5.0) && rating.fract() % 0.5 == 0.0
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PatchSpec {
    SetVotingInterval(Option<VoteInterval>),
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractPatch(Vec<PatchSpec>);

impl<'de> de::Deserialize<'de> for ContractPatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = ContractPatch;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                fmt.write_str("an object defining contract patches")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut vec = Vec::with_capacity(map.size_hint().unwrap_or(0));

                while let Some(k) = map.next_key()? {
                    match k {
                        "voting_interval" => {
                            vec.push(PatchSpec::SetVotingInterval(map.next_value()?))
                        }
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                            return Err(de::Error::unknown_field(k, &["voting_interval"]));
                        }
                    }
                }

                Ok(ContractPatch(vec))
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use near_sdk::{
        test_utils::{
            test_env::{alice, bob},
            VMContextBuilder,
        },
        testing_env,
    };

    use super::*;

    fn now() -> Timestamp {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as Timestamp
    }

    fn sys() -> AccountId {
        "nosedive_sys.near".parse().unwrap()
    }

    fn stage(account_id: AccountId) -> NoseDive {
        let context = VMContextBuilder::new()
            .current_account_id(sys())
            .signer_account_id(account_id.clone())
            .predecessor_account_id(account_id)
            .block_timestamp(now())
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
    fn rate_then_view() {
        stage(sys()).patch_state(ContractPatch(vec![PatchSpec::SetVotingInterval(None)]));

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
    fn lookup_timestaps() {
        use std::{thread, time};

        stage(alice()).register();
        stage(bob()).register();
        // --
        assert_eq!(
            stage(alice()).rating_timestamps(bob()),
            RatingTimestamps {
                they_rated_at: None,
                you_rated_at: None,
            }
        );
        // --
        let start = now();
        stage(bob()).rate(alice(), 4.5);
        let bob_alice = now();

        thread::sleep(time::Duration::from_secs(2));

        stage(alice()).rate(bob(), 4.5);
        let alice_bob = now();
        // --
        let ratings = stage(alice()).rating_timestamps(bob());
        assert!(matches!(
            ratings,
            RatingTimestamps {
                you_rated_at: Some(you_rated_at),
                they_rated_at: Some(they_rated_at)
            }
            if (start..=bob_alice).contains(&they_rated_at)
            && (bob_alice..=alice_bob).contains(&you_rated_at)
        ));
    }

    #[test]
    #[should_panic(
        expected = "only the account that deployed this contract is permitted to call this method"
    )]
    fn patch_auth_violation() {
        stage(alice()).patch_state(ContractPatch(vec![PatchSpec::SetVotingInterval(None)]));
    }

    #[test]
    fn patch_auth_pass() {
        stage(sys()).patch_state(ContractPatch(vec![PatchSpec::SetVotingInterval(None)]));
    }

    #[test]
    fn interval_pass() {
        use std::{thread, time};

        stage(sys()).patch_state(ContractPatch(vec![PatchSpec::SetVotingInterval(Some(
            VoteInterval {
                secs: 2,
                msg: "wait at least two seconds to be allowed to vote the same person again"
                    .to_string(),
            },
        ))]));

        stage(alice()).register();
        stage(bob()).register();
        // --
        stage(bob()).rate(alice(), 4.5);
        thread::sleep(time::Duration::from_secs(3));
        stage(bob()).rate(alice(), 4.5);
    }

    #[test]
    #[should_panic(expected = "you can't vote more than once in 5 minutes")]
    fn default_interval_violation() {
        stage(alice()).register();
        stage(bob()).register();
        // --
        stage(bob()).rate(alice(), 4.5);
        stage(bob()).rate(alice(), 4.5);
    }

    #[test]
    #[should_panic(expected = "all you had to do was wait a minute")]
    fn custom_interval_violation() {
        stage(sys()).patch_state(ContractPatch(vec![PatchSpec::SetVotingInterval(Some(
            VoteInterval {
                secs: 60,
                msg: "all you had to do was wait a minute".to_string(),
            },
        ))]));

        stage(alice()).register();
        stage(bob()).register();
        // --
        stage(bob()).rate(alice(), 4.5);
        stage(bob()).rate(alice(), 4.5);
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
