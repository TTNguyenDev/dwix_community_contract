use std::ops::Range;

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
    CheckRePost,
    CheckRePostInner { id: String },

    Topics,
    TopicsPosts,
    TopicsPostsInner { id: String },

    Communities,
    CommunitiesPosts,
    CommunitiesPostsInner { id: String },

    MemberInCommunites,
    MemberInCommunitesInner { id: String },
    Admins,

    PlaceIds,
    ChestsPerPlace,
    ChestsAtPlace { id: String },
    Chests,
}

pub fn valid_url(maybe_url: String) -> bool {
    Url::parse(&maybe_url).is_ok()
}

pub fn valid_post_id(_post_id: String) -> bool {
    true
}

pub fn calculate_rev_limit(len: u64, from_index: u64, limit: u64) -> Range<u64> {
    let from = if len > (limit + from_index) {
        len - limit - from_index
    } else {
        0
    };

    let to = if len > from_index {
        len - from_index
    } else {
        0
    };

    from..to
}
