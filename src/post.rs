use crate::*;
use near_sdk::serde::{Deserialize, Serialize};

pub type TokenId = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
pub enum PostType {
    Standard,
    Image { url: String },
    Video { url: String },
    RawbotNFT { token_id: TokenId },
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Post {
    pub id: PostId,
    pub account_id: AccountId,
    pub topic: Topic,
    pub title: String,
    pub body: String,
    pub post_type: PostType,
    pub time: U64,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PostStats {
    pub num_likes: i64,
    pub post_id: PostId,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VPost {
    Last(Post),
}

impl From<Post> for VPost {
    fn from(post: Post) -> Self {
        Self::Last(post)
    }
}

impl From<VPost> for Post {
    fn from(v_post: VPost) -> Self {
        match v_post {
            VPost::Last(post) => post,
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn post(
        &mut self,
        title: String,
        body: String,
        post_type: PostType,
        topic_id: TopicId,
    ) -> Post {
        assert!(self.topics.get(&topic_id).is_some(), "Not found your topic");

        assert!(
            title.len() <= MAX_TITLE_LENGTH,
            "Can not make a post title more than {} characters",
            MAX_TITLE_LENGTH
        );

        assert!(
            body.len() == IPFS_HASH_LENGTH,
            "Body should be an ipfs hash!!",
        );

        match post_type.clone() {
            PostType::Image { url } => assert!(valid_url(url), "Not valid url"),
            PostType::Video { url } => assert!(valid_url(url), "Not valid url"),
            PostType::RawbotNFT { token_id } => match token_id.parse::<u64>() {
                Err(e) => panic!("{}", e),
                _ => {}
            },
            _ => {}
        };

        let account_id = env::predecessor_account_id();
        let storage_update = self.new_storage_update(account_id.clone());
        let account = self.internal_get_account(&account_id);

        let block_timestamp = env::block_timestamp() / 1_000_000_000;
        let post_id = block_timestamp.to_string() + "_" + &env::predecessor_account_id();

        let post = Post {
            id: post_id.clone(),
            account_id: account_id.clone(),
            title,
            body,
            post_type,
            time: env::block_timestamp().into(),
            topic: self.topics.get(&topic_id).unwrap(),
        };

        let v_post = post.into();
        self.posts.insert(&post_id, &v_post);

        //Insert to user posts
        let mut user_posts = self
            .user_posts
            .get(&env::predecessor_account_id())
            .unwrap_or(UnorderedSet::new(StorageKey::UserPostsInner {
                id: env::predecessor_account_id(),
            }));

        user_posts.insert(&post_id);
        self.user_posts
            .insert(&env::predecessor_account_id(), &user_posts);

        //Insert to Topic posts
        let mut topics_posts = self
            .topics_posts
            .get(&account_id)
            .unwrap_or(UnorderedSet::new(StorageKey::TopicsPostsInner {
                id: account_id.clone(),
            }));

        topics_posts.insert(&post_id);
        self.topics_posts.insert(&account_id.clone(), &topics_posts);

        self.internal_set_account(&account_id, account);
        self.finalize_storage_update(storage_update);
        v_post.into()
    }

    pub fn delete_post(&mut self, post_id: PostId, post_owner: AccountId) {
        let signer = env::predecessor_account_id();

        let query_user = if self.is_admin(signer.clone()) {
            post_owner 
        } else {
            signer.clone()
        };

        let mut user_post = self
            .user_posts
            .get(&query_user)
            .expect("User doesn't have posts!");

        assert!(
            user_post.contains(&post_id) || self.is_admin(signer.clone()),
            "You are not the owner of this post"
        );

        //Delete this post
        self.posts.remove(&post_id);
        user_post.remove(&post_id);
        self.user_posts.insert(&query_user, &user_post);

        //Add post id to list
        self.deleted_posts.insert(&post_id);
    }

    //TODO: paging
    pub fn get_posts_by_account(&self, account_id: ValidAccountId) -> Vec<Post> {
        if let Some(posts) = self.user_posts.get(&account_id.into()) {
            return posts
                .iter()
                .map(|post_id| self.posts.get(&post_id).unwrap().into())
                .collect();
        }
        vec![]
    }

    pub fn get_num_posts_by_account(&self, account_id: ValidAccountId) -> u64 {
        if let Some(posts) = self.user_posts.get(&account_id.into()) {
            return posts.len();
        }
        0
    }

    pub fn get_posts_of_topic(&self, topic_id: TopicId) -> Vec<PostId> {
        if let Some(posts) = self.topics_posts.get(&topic_id) {
            return posts.to_vec();
        }
        vec![]
    }

    pub fn get_posts(&self, from_index: u64, limit: u64) -> Vec<Post> {
        let posts = self.posts.keys_as_vector();
        let from = if posts.len() > (limit + from_index) {
            posts.len() - limit - from_index
        } else {
            0
        };

        let to = if posts.len() > from_index {
            posts.len() - from_index
        } else {
            0
        };

        (from..to)
            .map(|index| {
                let post_id = posts.get(index).unwrap();
                self.posts.get(&post_id).unwrap().into()
            })
            .rev()
            .collect()
    }

    pub fn get_post_by_id(&self, post_id: PostId) -> Post {
        self.posts.get(&post_id).unwrap().into()
    }

    pub fn get_post_by_ids(&self, post_ids: Vec<PostId>) -> Vec<Post> {
        post_ids
            .iter()
            .map(|post_id| self.posts.get(&post_id).unwrap().into())
            .collect()
    }

    pub fn get_hot_posts(&self) -> Vec<PostStats> {
        self.posts_with_filter(ONE_DAY_UNIX_TIME)
    }

    pub fn get_trending_posts(&self) -> Vec<PostStats> {
        self.posts_with_filter(ONE_WEEK_UNIX_TIME)
    }

    fn posts_with_filter(&self, filter_duration: u64) -> Vec<PostStats> {
        let time_to_filter = env::block_timestamp() / 1000_000_000 - filter_duration;
        let mut un_sorted_vec: Vec<PostStats> = self
            .likes
            .keys()
            .filter(|k| {
                env::log(format!("{}", k).as_bytes());
                let id_splited: Vec<&str> = k.split("_").collect();

                let timestamp = id_splited[0].parse::<u64>().ok().unwrap();
                timestamp > time_to_filter
            })
            .map(|k| PostStats {
                post_id: k.clone(),
                num_likes: self.get_votes(k.clone()),
            })
            .collect();

        un_sorted_vec.sort_by(|a, b| b.num_likes.cmp(&a.num_likes));
        un_sorted_vec
    }
    pub fn test(&self) -> Vec<String> {
        self.likes.keys_as_vector().to_vec()
    }
}
