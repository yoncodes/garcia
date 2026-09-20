#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionRecord {
    pub misson_id: i32,
    pub total_num: i32,
    pub curr_num: i32,
    pub taken: bool,
    pub take_time: i32,
    pub mission_type: i32,
    pub created_at: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DungeonClearRecord {
    pub dungeon_type: i32,
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleAttrRecord {
    pub role_id: i32,
    pub mp: i32,
    pub ep: i32,
    pub hp: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormationPositionRecord {
    pub game_role_id: i32,
    pub key_num: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormationRecord {
    pub formation_id: i32,
    pub remark: String,
    pub positions: Vec<FormationPositionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRecord {
    pub id: i32,
    pub status: i32,
    pub picked_at: i64,
    pub progress: i32,
    pub total: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortRecord {
    pub id: i32,
    pub pass_cnt: i32,
    pub is_c: bool,
    pub is_f: bool,
    pub port_id: i32,
    pub s1: i32,
    pub s2: i32,
    pub s3: i32,
    pub t_cnt: i32,
    pub updated_at: i32,
    pub all_cnt: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleProgressRecord {
    pub role_id: i32,
    pub level: i32,
    pub exp: i32,
    pub user_partner_id: i64,
    pub position: i32,
    pub maid_qua: i32,
    pub skin_id: i32,
    pub appear_skill_key: i32,
    pub element: i32,
    pub element4call: i32,
    pub cur_mc: bool,
    pub awards: Vec<i32>,
    pub talents: Vec<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyTaskRecord {
    pub id: i32,
    pub taken: bool,
    pub progress: i32,
    pub total: i32,
}
