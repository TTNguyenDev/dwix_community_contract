use crate::*;
use near_sdk::collections::UnorderedSet;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub following: UnorderedSet<AccountId>,
    pub followers: UnorderedSet<AccountId>,
    pub related_conversations: UnorderedSet<MessageId>,
    pub message_pub_key: String,

    /// Personal Information
    pub avatar: String,
    pub thumbnail: String,
    pub bio: String,
    pub joined_communities: UnorderedSet<CommunityId>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VAccount {
    Last(Account),
}

impl From<Account> for VAccount {
    fn from(account: Account) -> Self {
        Self::Last(account)
    }
}

impl From<VAccount> for Account {
    fn from(v_account: VAccount) -> Self {
        match v_account {
            VAccount::Last(account) => account,
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountStats {
    pub num_followers: u64,
    pub num_following: u64,
    pub related_conversations: Vec<MessageId>,
    pub message_pub_key: String,

    pub avatar: String,
    pub thumbnail: String,
    pub bio: String,
}

impl From<Account> for AccountStats {
    fn from(account: Account) -> Self {
        Self {
            num_followers: account.followers.len(),
            num_following: account.following.len(),
            related_conversations: account.related_conversations.to_vec(),
            message_pub_key: account.message_pub_key,

            avatar: account.avatar,
            thumbnail: account.thumbnail,
            bio: account.bio,
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn joined_communities(
        &self,
        account_id: ValidAccountId,
        from_index: u64,
        limit: u64,
    ) -> Vec<WrappedCommunity> {
        let account = self.internal_get_account(&account_id.into());

        let keys = account.joined_communities.as_vector();

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| {
                let id = keys.get(index).expect("index of out bound");
                let mut community: WrappedCommunity = self.communities.get(&id).unwrap().into();

                community.posts_count =
                    if let Some(communites_posts) = self.communities_posts.get(&id) {
                        communites_posts.len() as u16
                    } else {
                        0
                    };

                community
            })
            .collect()
    }
    pub fn follow(&mut self, account_id: ValidAccountId) {
        let account_id = account_id.into();
        let from_account_id = env::predecessor_account_id();
        assert_ne!(
            &account_id, &from_account_id,
            "Can't follow your own account"
        );

        let storage_update = self.new_storage_update(from_account_id.clone());
        let mut from_account = self.internal_get_account(&from_account_id);
        assert!(
            from_account.following.insert(&account_id),
            "Already following this account"
        );
        self.internal_set_account(&from_account_id, from_account);

        let mut account = self.internal_get_account(&account_id);
        assert!(
            account.followers.insert(&from_account_id),
            "Already followed by your account"
        );
        self.internal_set_account(&account_id, account);
        self.finalize_storage_update(storage_update);
    }

    pub fn unfollow(&mut self, account_id: String) {
        let account_id = account_id.into();
        let from_account_id = env::predecessor_account_id();
        assert_ne!(
            &account_id, &from_account_id,
            "Can't unfollow your own account"
        );

        let storage_update = self.new_storage_update(from_account_id.clone());
        let mut from_account = self.internal_get_account(&from_account_id);
        assert!(
            from_account.following.remove(&account_id),
            "Not following this account"
        );
        self.internal_set_account(&from_account_id, from_account);

        let mut account = self.internal_get_account(&account_id);
        assert!(
            account.followers.remove(&from_account_id),
            "Not followed by your account"
        );
        self.internal_set_account(&account_id, account);
        self.finalize_storage_update(storage_update);
    }

    pub fn set_avatar(&mut self, avatar: String) {
        let account_id = env::predecessor_account_id();

        let storage_update = self.new_storage_update(account_id.clone());
        let mut account = self.internal_get_account(&account_id);
        account.avatar = avatar;
        self.internal_set_account(&account_id, account);
        self.finalize_storage_update(storage_update);
    }

    pub fn set_thumbnail(&mut self, thumbnail: String) {
        let account_id = env::predecessor_account_id();

        let storage_update = self.new_storage_update(account_id.clone());
        let mut account = self.internal_get_account(&account_id);
        account.thumbnail = thumbnail;
        self.internal_set_account(&account_id, account);
        self.finalize_storage_update(storage_update);
    }

    pub fn set_bio(&mut self, bio: String) {
        let account_id = env::predecessor_account_id();

        let storage_update = self.new_storage_update(account_id.clone());
        let mut account = self.internal_get_account(&account_id);
        account.bio = bio;
        self.internal_set_account(&account_id, account);

        self.finalize_storage_update(storage_update);
    }

    pub fn set_pub_key(&mut self, pub_key: String) {
        let account_id = env::predecessor_account_id();

        let storage_update = self.new_storage_update(account_id.clone());
        let mut account = self.internal_get_account(&account_id);
        account.message_pub_key = pub_key;
        self.internal_set_account(&account_id, account);

        self.finalize_storage_update(storage_update);
    }

    pub fn top_users(&self) -> Vec<AccountStats> {
        let mut result: Vec<AccountStats> = self
            .accounts
            .iter()
            .map(|(_id, item)| {
                let account: Account = item.into();
                account.into()
            })
            .collect();

        result.drain(..std::cmp::min(8, result.len())).collect()
    }

    pub fn get_followers(
        &self,
        account_id: ValidAccountId,
        from_index: u64,
        limit: u64,
    ) -> Vec<(AccountId, AccountStats)> {
        let account = self.internal_get_account(account_id.as_ref());
        self.get_account_range(account.followers.as_vector(), from_index, limit)
    }

    pub fn get_following(
        &self,
        account_id: ValidAccountId,
        from_index: u64,
        limit: u64,
    ) -> Vec<(AccountId, AccountStats)> {
        let account = self.internal_get_account(account_id.as_ref());
        self.get_account_range(account.following.as_vector(), from_index, limit)
    }

    pub fn get_account(&self, account_id: ValidAccountId) -> Option<AccountStats> {
        self.internal_get_account_optional(account_id.as_ref())
            .map(|a| a.into())
    }

    pub fn get_accounts(&self, from_index: u64, limit: u64) -> Vec<(AccountId, AccountStats)> {
        let account_ids = self.accounts.keys_as_vector();
        let accounts = self.accounts.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, account_ids.len()))
            .map(|index| {
                let account_id = account_ids.get(index).unwrap();
                let account: Account = accounts.get(index).unwrap().into();
                (account_id, account.into())
            })
            .collect()
    }

    pub fn get_num_accounts(&self) -> u64 {
        self.accounts.len()
    }

    pub fn get_accounts_with_ids(&self, account_ids: Vec<AccountId>) -> Vec<AccountStats> {
        account_ids
            .iter()
            .map(|id| {
                let account: Account = self.accounts.get(id).unwrap().into();
                account.into()
            })
            .collect()
    }
}
