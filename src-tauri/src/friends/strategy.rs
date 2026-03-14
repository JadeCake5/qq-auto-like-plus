use serde::Serialize;

use crate::db::models::FriendWithTags;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendLikeStrategy {
    pub user_id: i64,
    pub nickname: String,
    pub like_times: i32,
    pub priority_order: u8, // 0=high, 1=medium, 2=low
    pub auto_like: bool,
    pub auto_reply: bool,
}

fn priority_to_order(priority: &str) -> u8 {
    match priority {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 1,
    }
}

/// 解析单个好友的点赞策略（多标签取最高优先级）
pub fn resolve_friend_strategy(friend: &FriendWithTags, default_times: i32) -> FriendLikeStrategy {
    if friend.tags.is_empty() {
        return FriendLikeStrategy {
            user_id: friend.user_id,
            nickname: friend.nickname.clone(),
            like_times: default_times,
            priority_order: 1, // medium
            auto_like: true,
            auto_reply: true,
        };
    }

    // 找最高优先级标签
    let best_tag = friend
        .tags
        .iter()
        .min_by_key(|t| priority_to_order(&t.priority))
        .unwrap(); // safe: tags is non-empty

    // 任一标签 auto_like=false → 整体 false（"不赞"标签优先生效）
    let auto_like = friend.tags.iter().all(|t| t.auto_like);
    let auto_reply = friend.tags.iter().all(|t| t.auto_reply);

    // like_times 取最高优先级标签的值
    let like_times = best_tag.like_times.unwrap_or(default_times);

    FriendLikeStrategy {
        user_id: friend.user_id,
        nickname: friend.nickname.clone(),
        like_times,
        priority_order: priority_to_order(&best_tag.priority),
        auto_like,
        auto_reply,
    }
}

/// 构建点赞队列：过滤 + 按优先级排序 + 同级随机
pub fn build_like_queue(
    friends: Vec<FriendWithTags>,
    default_times: i32,
) -> Vec<FriendLikeStrategy> {
    use rand::seq::SliceRandom;

    let mut strategies: Vec<FriendLikeStrategy> = friends
        .iter()
        .map(|f| resolve_friend_strategy(f, default_times))
        .filter(|s| s.auto_like)
        .collect();

    // 按优先级排序
    strategies.sort_by_key(|s| s.priority_order);

    // 对同一优先级区间内随机打乱
    let mut rng = rand::rng();
    let mut start = 0;
    while start < strategies.len() {
        let current_priority = strategies[start].priority_order;
        let end = strategies[start..]
            .iter()
            .position(|s| s.priority_order != current_priority)
            .map(|p| start + p)
            .unwrap_or(strategies.len());
        strategies[start..end].shuffle(&mut rng);
        start = end;
    }

    strategies
}
