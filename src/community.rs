use super::*;
use near_sdk::collections::UnorderedSet;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WrappedCommunity {
    id: CommunityId,
    admin: ValidAccountId,
    thumbnail: String,
    avatar: String,
    name: String,
    created_time: U64,
    description: String,

    //stats
    pub posts_count: u16,
}

impl From<Community> for WrappedCommunity {
    fn from(community: Community) -> Self {
        WrappedCommunity {
            id: community.id,
            admin: community.admin,
            thumbnail: community.thumbnail,
            avatar: community.avatar,
            name: community.name,
            created_time: community.created_time,
            description: community.description,

            //stats
            posts_count: 0,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Community {
    id: CommunityId,
    admin: ValidAccountId,
    thumbnail: String,
    avatar: String,
    name: String,
    created_time: U64,
    description: String,
}

pub type CommunityId = String;

#[near_bindgen]
impl Contract {
    // make topic name is a id. because we want topic is unique
    pub fn new_community(
        &mut self,
        thumbnail: Option<String>,
        avatar: Option<String>,
        name: String,
        description: String,
    ) -> CommunityId {
        let community_id = name.to_lowercase().replace(" ", "_");

        assert!(
            name.len() <= MAX_TITLE_LENGTH,
            "Can not make a post title more than {} characters",
            MAX_TITLE_LENGTH
        );

        assert!(
            description.len() <= MAX_BODY_LENGTH,
            "Can not make a post body more than {} characters",
            MAX_BODY_LENGTH
        );

        assert!(
            !self.communities.get(&community_id.clone()).is_some(),
            "Community already exists"
        );

        let account_id = env::predecessor_account_id();
        let storage_update = self.new_storage_update(account_id.clone());

        let mut prefix = Vec::with_capacity(33);
        prefix.push(b'c');
        prefix.extend(env::sha256(env::predecessor_account_id().as_bytes()));

        let community = Community {
            id: community_id.clone(),
            name,
            admin: ValidAccountId::try_from(account_id.to_string()).unwrap(),
            thumbnail: thumbnail.unwrap_or("".to_string()),
            avatar: avatar.unwrap_or("".to_string()),
            // members: UnorderedSet::new(prefix),
            // thumbnail: topic_thumbnail,
            created_time: env::block_timestamp().into(),
            description,
        };

        let mut members = UnorderedSet::new(StorageKey::MemberInCommunitesInner {
            id: community_id.clone(),
        });
        members.insert(&env::predecessor_account_id());
        self.members_in_communites
            .insert(&community_id.clone(), &members);

        //add community Id
        let mut user: Account = self
            .accounts
            .get(&account_id)
            .expect("User not found")
            .into();
        user.joined_communities.insert(&community_id);
        self.accounts.insert(&account_id, &user.into());

        self.communities.insert(&community_id.clone(), &community);
        self.finalize_storage_update(storage_update);
        community_id
    }

    pub fn community_by_id(&self, community_id: CommunityId) -> WrappedCommunity {
        let community = self
            .communities
            .get(&community_id)
            .expect("Not found your community");

        WrappedCommunity::from(community)
    }

    //Join a community / left a community
    pub fn join_community(&mut self, community_id: CommunityId) {
        let mut members = self
            .members_in_communites
            .get(&community_id)
            .expect("Not found your community");

        // let new_member = ValidAccountId::try_from(env::predecessor_account_id()).unwrap();
        assert!(
            !members.contains(&env::predecessor_account_id()),
            "You're already a member of this community"
        );

        //add community Id
        let mut user: Account = self
            .accounts
            .get(&env::predecessor_account_id())
            .expect("User not found")
            .into();
        user.joined_communities.insert(&community_id);
        self.accounts
            .insert(&env::predecessor_account_id(), &user.into());

        members.insert(&env::predecessor_account_id());
        self.members_in_communites.insert(&community_id, &members);
    }

    pub fn leave_community(&mut self, community_id: CommunityId) {
        let community = self
            .communities
            .get(&community_id)
            .expect("Not found your community");

        let mut members = self
            .members_in_communites
            .get(&community_id)
            .expect("Not found your community");

        let new_member = ValidAccountId::try_from(env::predecessor_account_id()).unwrap();
        assert!(
            community.admin != new_member,
            "Admin can not leave community"
        );
        assert!(
            members.contains(&env::predecessor_account_id()),
            "You're not a member of this community"
        );

        let mut user: Account = self
            .accounts
            .get(&new_member.clone().into())
            .expect("User not found")
            .into();
        user.joined_communities.remove(&community_id);
        self.accounts
            .insert(&new_member.clone().into(), &user.into());

        members.remove(&env::predecessor_account_id());
        self.members_in_communites.insert(&community_id, &members);
    }

    pub fn already_joined(&self, community_id: CommunityId, account_id: ValidAccountId) -> bool {
        let members = self
            .members_in_communites
            .get(&community_id)
            .expect("Not found your community");
        members.contains(&account_id.into())
    }

    //Create communities posts
    pub fn community_post(
        &mut self,
        title: String,
        body: String,
        post_type: PostType,
        topic_id: TopicId,
        community_id: CommunityId,
    ) -> Post {
        assert!(self.topics.get(&topic_id).is_some(), "Not found your topic");

        assert!(
            title.len() <= MAX_TITLE_LENGTH,
            "Can not make a post title more than {} characters",
            MAX_TITLE_LENGTH
        );

        assert!(
            body.len() <= MAX_BODY_LENGTH,
            "Can not make a post body more than {} characters",
            MAX_BODY_LENGTH
        );

        let members = self
            .members_in_communites
            .get(&community_id)
            .expect("Not found your community");

        assert!(
            members.contains(&env::predecessor_account_id()),
            "You're not a member of this community"
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

        let block_height = env::block_index();
        let block_timestamp = env::block_timestamp() / 1_000_000_000;
        let post_id = block_height.to_string()
            + "_"
            + &block_timestamp.to_string()
            + "_"
            + &env::predecessor_account_id();

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
        let mut posts = self
            .communities_posts
            .get(&community_id)
            .unwrap_or(UnorderedMap::new(StorageKey::CommunitiesPostsInner {
                id: community_id.clone() + &env::block_index().to_string(),
            }));

        posts.insert(&post_id, &v_post);
        self.communities_posts.insert(&community_id, &posts);

        // account.num_posts += 1;
        // account.last_post_height = block_height;
        self.internal_set_account(&account_id, account);
        self.finalize_storage_update(storage_update);
        v_post.into()
    }

    pub fn delete_community_post(&mut self, post_id: PostId, community_id: CommunityId) {
        let owner = env::predecessor_account_id();
        let community = self
            .communities
            .get(&community_id)
            .expect("Community not found");

        let mut posts = self
            .communities_posts
            .get(&community_id)
            .expect("Community's post not found");

        let post: Post = posts.get(&post_id).expect("Post not found").into();

        assert!(
            owner == post.account_id || owner == community.admin.to_string() || self.is_admin(owner),
            "You don't have permission to delete"
        );

        //Delete this post
        posts.remove(&post_id);
        self.communities_posts.insert(&community_id, &posts);

        //Add post id to list
        self.deleted_posts.insert(&post_id);
    }

    pub fn get_deleted_posts(&self) -> Vec<PostId> {
        self.deleted_posts.to_vec()
    }

    //TODO: Paging
    pub fn get_communities(&self, from_index: u64, limit: u64) -> Vec<WrappedCommunity> {
        let keys = self.communities.keys_as_vector();
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

    pub fn get_members(&self, community_id: CommunityId) -> Vec<(AccountId, AccountStats)> {
        self.members_in_communites
            .get(&community_id)
            .expect("No members found")
            .iter()
            .map(|id| (id.clone(), self.internal_get_account(&id).into()))
            .collect()
    }

    pub fn get_community_posts(
        &self,
        community_id: CommunityId,
        from_index: u64,
        limit: u64,
    ) -> Vec<Post> {
        let posts = self
            .communities_posts
            .get(&community_id)
            .unwrap_or(UnorderedMap::new(StorageKey::CommunitiesPostsInner {
                id: community_id.clone() + &env::block_index().to_string(),
            }));

        let keys = posts.keys_as_vector();

        let from = if keys.len() > (limit + from_index) {
            keys.len() - limit - from_index
        } else {
            0
        };

        let to = if keys.len() > from_index {
            keys.len() - from_index
        } else {
            0
        };

        (from..to)
            .map(|index| {
                let id = keys.get(index).expect("index of out bound");
                posts.get(&id).unwrap().into()
            })
            .rev()
            .collect()
    }

    pub fn get_community_post_with_id(&self, community_id: CommunityId, post_id: PostId) -> Post {
        let posts = self
            .communities_posts
            .get(&community_id)
            .expect("Community not found");

        posts.get(&post_id).expect("Post not found").into()
    }

    pub fn set_community_thumbnail(&mut self, thumbnail: String, community_id: CommunityId) {
        let account_id = env::predecessor_account_id();

        let mut community = self
            .communities
            .get(&community_id)
            .expect("Community not found");

        assert!(
            account_id == community.admin.to_string(),
            "You're not the admin of this community"
        );

        let storage_update = self.new_storage_update(account_id.clone());
        community.thumbnail = thumbnail;
        self.communities.insert(&community_id, &community);
        self.finalize_storage_update(storage_update);
    }

    pub fn set_community_avatar(&mut self, avatar: String, community_id: CommunityId) {
        let account_id = env::predecessor_account_id();

        let mut community = self
            .communities
            .get(&community_id)
            .expect("Community not found");

        assert!(
            account_id == community.admin.to_string(),
            "You're not the admin of this community"
        );

        let storage_update = self.new_storage_update(account_id.clone());
        community.avatar = avatar;
        self.communities.insert(&community_id, &community);
        self.finalize_storage_update(storage_update);
    }

    pub fn set_community_bio(&mut self, description: String, community_id: CommunityId) {
        let account_id = env::predecessor_account_id();

        let mut community = self
            .communities
            .get(&community_id)
            .expect("Community not found");

        assert!(
            account_id == community.admin.to_string(),
            "You're not the admin of this community"
        );

        let storage_update = self.new_storage_update(account_id.clone());
        community.description = description;
        self.communities.insert(&community_id, &community);
        self.finalize_storage_update(storage_update);
    }

    pub fn top_community(&self) -> Vec<WrappedCommunity> {
        let mut result: Vec<WrappedCommunity> = self
            .communities
            .iter()
            .map(|(_id, item)| item.into())
            .collect();

        result.drain(..std::cmp::min(8, result.len())).collect()
    }
}
