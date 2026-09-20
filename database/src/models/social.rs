#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FriendLink {
    pub uid: i64,
    pub has_gift: bool,
    pub gift_sent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SocialState {
    pub friends: Vec<FriendLink>,
    pub pending_approvals: Vec<i64>,
    pub claimed_gifts: i32,
}
