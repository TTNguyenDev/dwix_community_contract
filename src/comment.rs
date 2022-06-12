use super::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Comment {
    owner: AccountId,
    body: String,
    time: U64,
}

#[near_bindgen]
impl Contract {
    pub fn comment(&mut self, post_id: PostId, body: String) -> Comment {
        assert!(
            body.len() <= IPFS_HASH_LENGTH,
            "Body should be an ipfs hash!!",
        );

        let account_id = env::predecessor_account_id();
        let storage_update = self.new_storage_update(account_id.clone());

        let comment = Comment {
            owner: account_id,
            body,
            time: env::block_timestamp().into(),
        };

        if let Some(mut values) = self.comments.get(&post_id) {
            values.push(&comment);
            self.comments.insert(&post_id, &values);
        } else {
            let mut comment_key = vec![b'c'];

            let hash = env::sha256(post_id.as_bytes());
            comment_key.extend_from_slice(&hash);

            let mut values: Vector<Comment> = Vector::new(comment_key);

            values.push(&comment);
            self.comments.insert(&post_id, &values);
        }

        self.finalize_storage_update(storage_update);
        comment
    }

    pub fn edit_comment(&mut self, post_id: PostId, comment_index: u64, body: String) -> Comment {
        let account_id = env::predecessor_account_id();
        assert!(
            body.len() <= IPFS_HASH_LENGTH,
            "Body should be an ipfs hash!!",
        );

        let storage_update = self.new_storage_update(account_id.clone());
        let mut comments = self
            .comments
            .get(&post_id)
            .expect("Not found comments in post");

        let mut comment = comments.get(comment_index).expect("Out of bound");
        assert!(
            comment.owner == account_id,
            "You don't have permission to edit this comment"
        );

        comment.body = body;
        comment.time = env::block_timestamp().into();
        comments.replace(comment_index, &comment);
        self.comments.insert(&post_id, &comments);

        self.finalize_storage_update(storage_update);
        comment.into()
    }

    pub fn get_comments(&self, post_id: PostId, from_index: u64, limit: u64) -> Vec<Comment> {
        let comments = self.comments.get(&post_id).unwrap_or(Vector::new(b"v"));

        let from = if comments.len() > (limit + from_index) {
            comments.len() - limit - from_index
        } else {
            0
        };

        let to = if comments.len() > from_index {
            comments.len() - from_index
        } else {
            0
        };

        (from..to)
            .map(|index| comments.get(index).unwrap())
            .rev()
            .collect()
    }

    pub fn get_num_post_comments(&self, post_id: PostId) -> u64 {
        self.comments
            .get(&post_id)
            .unwrap_or(Vector::new(b"v"))
            .len()
    }
}
