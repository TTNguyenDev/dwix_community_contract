use crate::*;

#[near_bindgen]
impl Contract {
    pub fn add_admin(&mut self, account_id: AccountId) -> bool {
        let caller_id = env::predecessor_account_id();
        let contract_id = env::current_account_id();
        assert!(
            caller_id == "neilharan.testnet" 
            || caller_id == contract_id,
            "You not have permission to give admin rights"); 
        assert!(!self.is_admin(account_id.clone()), "This account already have admin rights");
        self.admins.insert(&account_id)
    }

    pub fn remove_admin(&mut self, account_id: AccountId) -> bool {
        let caller_id = env::predecessor_account_id();
        let contract_id = env::current_account_id();
        assert!(
            caller_id == "neilharan.testnet" 
            || caller_id == contract_id,
            "You not have permission to remove admin rights"); 
        assert!(self.is_admin(account_id.clone()), "This account not have admin rights");
        self.admins.remove(&account_id)
    }

    pub fn is_admin(&self, account_id: AccountId) -> bool {
        self.admins.contains(&account_id)
    }

    // //NOTE: Migrate function
    // #[private]
    // #[init(ignore_state)]
    // pub fn migrate(&self) -> Self {
    //     assert!(self.is_admin(env::predecessor_account_id()), "You don't have permission to migrate this contract!");
    //     let old_state: OldContract = env::state_read().expect("failed");
    //     Self {
    //         storage_accounts: old_state.storage_accounts,
    //         accounts: old_state.accounts,
    //         posts: old_state.posts,
    //         user_posts: old_state.user_posts,
    //         deleted_posts: old_state.deleted_posts,
    //         messages: old_state.messages,
    //         likes: old_state.likes,
    //         comments: old_state.comments,
    //         topics: old_state.topics,
    //         topics_posts: old_state.topics_posts,
    //         communities: old_state.communities,
    //         communities_posts: old_state.communities_posts,
    //         members_in_communites: old_state.members_in_communites,
    //         storage_account_in_bytes: old_state.storage_account_in_bytes,
    //         admins: LookupSet::new(StorageKey::Admins),
    //     }
    // }
}
