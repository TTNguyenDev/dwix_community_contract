use crate::*;
use url::Url;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    StorageAccount,
    Accounts,

    Posts,
    UserPosts,
    DeletedPosts,
    UserPostsInner { id: String },

    Messages,
    Likes,
    Commnets,

    Topics,
    TopicsPosts,
    TopicsPostsInner { id: String },

    Communities,
    CommunitiesPosts,
    CommunitiesPostsInner { id: String },

    MemberInCommunites,
    MemberInCommunitesInner { id: String },
    Admins,
}

pub fn valid_url(maybe_url: String) -> bool {
    return Url::parse(&maybe_url).is_ok();
}
