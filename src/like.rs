use super::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
pub enum VoteStatus {
    UpVote,
    // DownVote,
    Default,
}

#[near_bindgen]
impl Contract {
    pub fn upvote(&mut self, post_id: PostId) -> bool {
        let account_id = env::predecessor_account_id();

        let storage_update = self.new_storage_update(account_id.clone());

        if let Some(mut likes_map) = self.likes.get(&post_id) {
            if let Some(vote_value) = likes_map.get(&account_id) {
                assert!(vote_value == 0, "You already upvote!");
                likes_map.insert(&account_id, &1);
            } else {
                likes_map.insert(&account_id, &1);
            }
            self.likes.insert(&post_id, &likes_map);

            true
        } else {
            let mut like_key = vec![b'l'];

            let hash = env::sha256(post_id.as_bytes());
            like_key.extend_from_slice(&hash);

            let mut values = UnorderedMap::new(like_key);

            values.insert(&account_id, &1);
            self.likes.insert(&post_id, &values);
            self.finalize_storage_update(storage_update);

            true
        }
    }

    // pub fn downvote(&mut self, post_id: PostId) -> bool {
    //     let account_id = env::predecessor_account_id();
    //
    //     let storage_update = self.new_storage_update(account_id.clone());
    //
    //     if let Some(mut likes_map) = self.likes.get(&post_id) {
    //         if let Some(vote_value) = likes_map.get(&account_id) {
    //             assert!(vote_value == 1, "You already downvote!");
    //             likes_map.insert(&account_id, &0);
    //         } else {
    //             likes_map.insert(&account_id, &0);
    //         }
    //         self.likes.insert(&post_id, &likes_map);
    //
    //         true
    //     } else {
    //         let mut like_key = vec![b'l'];
    //
    //         let hash = env::sha256(post_id.as_bytes());
    //         like_key.extend_from_slice(&hash);
    //
    //         let mut values = UnorderedMap::new(like_key);
    //
    //         values.insert(&account_id, &0);
    //         self.likes.insert(&post_id, &values);
    //         self.finalize_storage_update(storage_update);
    //
    //         true
    //     }
    // }

    pub fn unvote(&mut self, post_id: PostId) {
        let account_id = env::predecessor_account_id();

        let storage_update = self.new_storage_update(account_id.clone());

        if let Some(mut likes_map) = self.likes.get(&post_id) {
            likes_map.remove(&account_id);
            self.likes.insert(&post_id, &likes_map);
            self.finalize_storage_update(storage_update);
        }
    }

    pub fn get_votes(&self, post_id: PostId) -> i64 {
        let upvotes = self
            .likes
            .get(&post_id)
            .unwrap_or_else(|| UnorderedMap::new(b"x"))
            .iter()
            .filter(|(_k, v)| v == &1)
            .count() as i64;
        2 * upvotes
            - self
                .likes
                .get(&post_id)
                .unwrap_or_else(|| UnorderedMap::new(b"x"))
                .len() as i64
    }

    // pub fn get_downvotes(&self, post_id: PostId) -> u64 {
    //     self.likes
    //         .get(&post_id)
    //         .unwrap_or(UnorderedMap::new(b"x"))
    //         .iter()
    //         .filter(|(_k, v)| v == &0)
    //         .collect::<Vec<(AccountId, u8)>>()
    //         .len() as u64
    // }

    pub fn vote_status(&self, post_id: PostId, account_id: AccountId) -> VoteStatus {
        if let Some(values) = self.likes.get(&post_id) {
            let vote_status = values.get(&account_id);
            if vote_status == Some(1) {
                VoteStatus::UpVote
            } else {
                VoteStatus::Default
            }
        } else {
            VoteStatus::Default
        }
    }


    
}
