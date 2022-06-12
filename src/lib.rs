use std::str;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, Vector, LookupSet};
use near_sdk::json_types::{ValidAccountId, U64};
use near_sdk::serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use near_sdk::{
    env, near_bindgen, setup_alloc, AccountId, Balance, BlockHeight, BorshStorageKey, Promise,
    StorageUsage,
};

pub use crate::account::*;
pub use crate::comment::*;
pub use crate::community::*;
pub use crate::internal_account::*;
pub use crate::like::*;
pub use crate::post::*;
pub use crate::private_message::*;
pub use crate::storage::*;
pub use crate::topic::*;
pub use crate::utils::*;
pub use crate::admin::*;

/// CONSTANTS
pub use crate::constant::*;

type PostId = String;

mod account;
mod comment;
mod community;
mod constant;
mod internal_account;
mod like;
mod post;
mod private_message;
mod storage;
mod topic;
mod utils;
mod admin;

setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub storage_accounts: LookupMap<AccountId, StorageAccount>,
    pub accounts: UnorderedMap<AccountId, VAccount>,

    pub posts: UnorderedMap<PostId, VPost>,
    pub user_posts: LookupMap<AccountId, UnorderedSet<PostId>>,
    pub deleted_posts: UnorderedSet<PostId>,

    pub messages: LookupMap<MessageId, PrivateMessage>,
    pub likes: UnorderedMap<PostId, UnorderedMap<AccountId, u8>>, //get for Hot page
    pub comments: LookupMap<PostId, Vector<Comment>>, //Should use hashmap to store comment

    pub topics: UnorderedMap<TopicId, Topic>,
    pub topics_posts: LookupMap<TopicId, UnorderedSet<PostId>>,

    pub communities: UnorderedMap<CommunityId, Community>,
    pub communities_posts: UnorderedMap<CommunityId, UnorderedMap<PostId, VPost>>,

    pub members_in_communites: UnorderedMap<CommunityId, UnorderedSet<AccountId>>,
    pub storage_account_in_bytes: StorageUsage,
    pub admins: LookupSet<AccountId>,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic(b"Contract should be initialized before usage");
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut this = Self {
            storage_accounts: LookupMap::new(StorageKey::StorageAccount),
            accounts: UnorderedMap::new(StorageKey::Accounts),

            posts: UnorderedMap::new(StorageKey::Posts),
            user_posts: LookupMap::new(StorageKey::UserPosts),
            deleted_posts: UnorderedSet::new(StorageKey::DeletedPosts),

            messages: LookupMap::new(StorageKey::Messages),
            likes: UnorderedMap::new(StorageKey::Likes),
            comments: LookupMap::new(StorageKey::Commnets),

            topics: UnorderedMap::new(StorageKey::Topics),
            topics_posts: LookupMap::new(StorageKey::TopicsPosts),

            communities: UnorderedMap::new(StorageKey::Communities),
            communities_posts: UnorderedMap::new(StorageKey::CommunitiesPosts),

            members_in_communites: UnorderedMap::new(StorageKey::MemberInCommunites),
            storage_account_in_bytes: 0,
            admins: LookupSet::new(StorageKey::Admins),
        };

        this.measure_storage_account_in_bytes();
        this
    }

    fn measure_storage_account_in_bytes(&mut self) {
        let account_id = LONGEST_ACCOUNT_ID.to_string();
        assert_eq!(account_id.len(), MAX_ACCOUNT_ID_LENGTH);

        let initial_storage = env::storage_usage();
        self.storage_accounts.insert(
            &account_id,
            &StorageAccount {
                balance: 0,
                used_bytes: 0,
            },
        );

        self.storage_account_in_bytes = env::storage_usage() - initial_storage;
        self.storage_accounts.remove(&account_id);
    }
}
