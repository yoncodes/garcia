#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipRecord {
    pub equip_id: i32,
    pub exp: i32,
    pub user_equip_id: i64,
    pub in_group: i32,
    pub locked: i32,
    pub level: i32,
    pub equiped_role: i32,
    pub pos: i32,
    pub main_words_id: i32,
    pub deputy_words_id: Vec<i32>,
    pub quality: i32,
    pub random_words_id: Vec<i32>,
    pub planned_words_id: Vec<i32>,
    pub minnum: i32,
    pub tmp_word: i32,
    pub tmp_word_idx: i32,
    pub creat_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentGroupRecord {
    pub id: i64,
    pub game_role_id: i32,
    pub group_name: String,
    pub user_equip_ids: Vec<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamCoreRecord {
    pub pos: i32,
    pub id: i64,
    pub locked: bool,
    pub equipped_id: i64,
    pub core_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamEquipRecord {
    pub equipped_formation_id: i32,
    pub id: i64,
    pub locked: bool,
    pub team_equip_id: i32,
    pub level: i32,
    pub exp: i32,
    pub main_words_id: i32,
    pub pos_num: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillStoneRecord {
    pub pos: i32,
    pub user_stone_id: i64,
    pub locked: bool,
    pub equipped_role: i32,
    pub stone_id: i32,
    pub quality: i32,
}
