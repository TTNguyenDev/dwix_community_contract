use super::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Topic {
    id: TopicId,
    admin: ValidAccountId,
    // thumbnail: String,
    name: String,
    created_time: U64,
    description: String,
}

pub type TopicId = String;

#[near_bindgen]
impl Contract {
    // make topic name is a id. because we want topic is unique
    pub fn new_topic(
        &mut self,
        topic_name: String,
        // topic_thumbnail: String,
        topic_desc: String,
    ) -> bool {
        let topic_id = topic_name.to_lowercase().replace(" ", "_");

        assert!(
            topic_name.len() <= MAX_TITLE_LENGTH,
            "Can not make a post title more than {} characters",
            MAX_TITLE_LENGTH
        );

        assert!(
            topic_desc.len() <= MAX_BODY_LENGTH,
            "Can not make a post body more than {} characters",
            MAX_BODY_LENGTH
        );

        assert!(
            !self.topics.get(&topic_id.clone()).is_some(),
            "Topic already exists"
        );

        let account_id = env::predecessor_account_id();
        let storage_update = self.new_storage_update(account_id.clone());

        let topic = Topic {
            id: topic_id.clone(),
            name: topic_name,
            admin: ValidAccountId::try_from(account_id.to_string()).unwrap(),
            // thumbnail: topic_thumbnail,
            created_time: env::block_timestamp().into(),
            description: topic_desc,
        };

        self.topics.insert(&topic_id.clone(), &topic);
        self.finalize_storage_update(storage_update);
        return true;
    }

    pub fn topics(&self) -> Vec<Topic> {
        self.topics.values().collect()
    }
}
