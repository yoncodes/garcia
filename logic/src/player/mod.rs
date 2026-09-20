use std::collections::{BTreeMap, HashMap};

use common::{config::AccountDefaults, time::ServerTime};
use configs::GameTables;
use protocol::pbcommon::{
    AlbumStatus, DcNetDataAchi, DcNetDataAct7Day, DcNetDataActivityBossInfo,
    DcNetDataActivityOpenInfo, DcNetDataAlbums, DcNetDataArchiveInfo, DcNetDataAttrUpData,
    DcNetDataBattlePass, DcNetDataBattlePassTask, DcNetDataBossRankElem, DcNetDataBossRushBase,
    DcNetDataChallenge, DcNetDataCheckInfo, DcNetDataCollection, DcNetDataCollectionPlaceInfo,
    DcNetDataCollectionSuitReward, DcNetDataDungeonSweepInfo, DcNetDataEmail, DcNetDataEquip,
    DcNetDataEquipSetRes, DcNetDataEquipsGroup, DcNetDataFavor, DcNetDataFeat, DcNetDataFormation,
    DcNetDataFormationPos, DcNetDataGachaData, DcNetDataGachaLog, DcNetDataGachaResInfo,
    DcNetDataInteractObj, DcNetDataItem, DcNetDataLocal, DcNetDataMallGoods, DcNetDataMissonInfo,
    DcNetDataMonsterManual4Port, DcNetDataMonsterPointGroup, DcNetDataParamsSettlement,
    DcNetDataPartner, DcNetDataPartnerReward, DcNetDataPort, DcNetDataProfileAvatar,
    DcNetDataProfileCard, DcNetDataProfileFrame, DcNetDataProfileTitle, DcNetDataRedDot,
    DcNetDataRegionCoinLimit, DcNetDataReward, DcNetDataRewardsRes, DcNetDataRoleAttrInfo,
    DcNetDataRoleBasicInfo, DcNetDataRoleReward, DcNetDataRolesRolesDetail, DcNetDataRouge,
    DcNetDataRougeInfo, DcNetDataSellRow, DcNetDataSkillStone, DcNetDataSms, DcNetDataTBoxInfo,
    DcNetDataTakeRewardRes, DcNetDataTalent, DcNetDataTaskCycle, DcNetDataTaskStatus,
    DcNetDataTeamCore, DcNetDataTeamEquip, DcNetDataTeamEquipDetail, DcNetDataUseItem,
    DcNetDataVersionChallengeInfo, DcNetDataVersionTask, FeatStatus, PaidStatus, Reddot, RoleInfo,
    TaskStatus,
};

mod activity;
mod activity_boss;
mod activity_challenge;
mod battle_pass;
mod boss_rush;
mod collection;
mod communication;
mod crafting;
mod daily;
mod disassembly;
mod dungeon;
mod economy;
mod equipment;
mod errors;
mod exploration;
mod favor;
mod gacha;
mod heat;
mod identity;
mod initialization;
mod inventory;
mod more_team_challenge;
mod partner;
mod persistence;
mod progression;
mod rift;
mod role_attr;
mod role_customization;
mod rouge;
mod skillstone;
mod sms;
mod social;
mod support;
mod tasks;
mod team_core;
mod team_equip;
mod trial;
mod version_challenge;
mod world;

use activity_boss::ActivityBossState;
use battle_pass::BattlePassState;
use boss_rush::BossRushState;
pub use crafting::CraftOutcome;
pub use disassembly::DisassemblyOutcome;
pub use economy::{
    ShopPurchaseOutcome, active_table_window, mall_goods_runtime, mall_goods_runtime_with_override,
    mall_group_runtime, mall_group_runtime_with_override, shop_goods_runtime, shop_runtime,
};
pub use errors::*;
pub use heat::{HeatExchangeOutcome, HeatInfoOutcome};
pub use inventory::ItemUseOutcome;
use more_team_challenge::MoreTeamChallengeState;
pub use progression::ProgressionOutcome;
pub use rift::RiftState;
pub use support::gacha_availability;
use support::*;

const PLAYER_EXP_ITEM_ID: i32 = 100_100_004;

pub struct RougeFinishOutcome {
    pub rouge_score: i32,
    pub reward: DcNetDataTakeRewardRes,
    pub items: Vec<DcNetDataItem>,
    pub rouge_info: DcNetDataRougeInfo,
}

#[derive(Debug)]
pub struct RoleRankOutcome {
    pub game_role_id: i32,
    pub position: i32,
    pub items: Vec<DcNetDataItem>,
    pub talents: Vec<DcNetDataTalent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GachaState {
    pub gacha_id: i32,
    pub all_count: i32,
    pub gacha_count_10: i32,
    pub reward_cnt: i32,
    pub reward_num: i32,
    pub taken_new_reward: bool,
    pub had_take_reward: bool,
}

pub struct GachaOutcome {
    pub reward_list: Vec<DcNetDataGachaResInfo>,
    pub gacha_data: DcNetDataGachaData,
    pub remains: Vec<DcNetDataItem>,
    pub coin: Option<DcNetDataItem>,
    pub events: Vec<GachaEvent>,
    pub battle_pass_updates: Vec<DcNetDataBattlePassTask>,
}

pub struct GachaClaimOutcome {
    pub reward: DcNetDataTakeRewardRes,
    pub gacha_data: DcNetDataGachaData,
    pub events: Vec<GachaEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GachaEvent {
    Achievement(DcNetDataAchi),
    Archive(DcNetDataArchiveInfo),
    ProfileAvatar(DcNetDataProfileAvatar),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GachaAvailability {
    Permanent,
    Active { remaining_seconds: i32 },
    Upcoming,
    Expired,
    InvalidSchedule,
}

pub struct RoleLevelOutcome {
    pub user_up_data: DcNetDataAttrUpData,
    pub remain: Vec<DcNetDataItem>,
    pub talents: Vec<DcNetDataTalent>,
    pub battle_pass_updates: Vec<DcNetDataBattlePassTask>,
    pub achievement_updates: Vec<DcNetDataAchi>,
    pub seven_day_updates: Vec<DcNetDataAct7Day>,
    pub daily_updates: Vec<DcNetDataTaskCycle>,
}

pub struct RoleMaxOutcome {
    pub role: DcNetDataRolesRolesDetail,
    pub namecard: Option<DcNetDataProfileCard>,
    pub changed: bool,
}

pub struct PartnerChangeOutcome {
    pub partner: DcNetDataPartner,
    pub role_info: DcNetDataRoleBasicInfo,
    pub unset_role_info: Option<DcNetDataRoleBasicInfo>,
}

pub struct PartnerUnsetOutcome {
    pub partner: DcNetDataPartner,
    pub role_info: DcNetDataRoleBasicInfo,
}

pub struct PartnerSwapOutcome {
    pub role_info1: DcNetDataRoleBasicInfo,
    pub role_info2: DcNetDataRoleBasicInfo,
}

#[derive(Debug)]
pub struct PartnerLevelOutcome {
    pub partner: DcNetDataPartner,
    pub cost_remain: Vec<DcNetDataItem>,
    pub remain: Vec<DcNetDataItem>,
}

#[derive(Debug)]
pub struct PartnerResonanceOutcome {
    pub partner: DcNetDataPartner,
    pub consumed_partners: Vec<i64>,
    pub remains: Vec<DcNetDataItem>,
}

#[derive(Debug)]
pub struct TaskCompleteOutcome {
    pub current: DcNetDataTaskStatus,
    pub changed: Vec<DcNetDataTaskStatus>,
    pub achievement_updates: Vec<DcNetDataAchi>,
    pub archive_updates: Vec<DcNetDataArchiveInfo>,
    pub album_updates: Vec<DcNetDataAlbums>,
    pub sms_updates: Vec<DcNetDataSms>,
    pub interact_updates: Vec<DcNetDataInteractObj>,
    pub rewards: DcNetDataTakeRewardRes,
    pub level_up_data: Option<DcNetDataAttrUpData>,
    pub region_reward: Option<(i32, DcNetDataItem)>,
    pub world_level: Option<i32>,
    pub mission_updates: Vec<DcNetDataMissonInfo>,
    pub feat_updates: Vec<DcNetDataFeat>,
}

pub struct TaskProgressOutcome {
    pub progress: DcNetDataTaskStatus,
    pub completion: Option<TaskCompleteOutcome>,
}

pub struct InteractionOutcome {
    pub object: DcNetDataInteractObj,
    pub reward: Option<DcNetDataTakeRewardRes>,
}

pub struct FeatureUnlockOutcome {
    pub requirements: Vec<DcNetDataTaskStatus>,
    pub completions: Vec<TaskCompleteOutcome>,
    pub ports: Vec<DcNetDataPort>,
    pub level_up_data: Option<DcNetDataAttrUpData>,
    pub features: Vec<DcNetDataFeat>,
}

pub struct TrialDoneOutcome {
    pub trial_id: i32,
    pub reward: DcNetDataTakeRewardRes,
    pub local: Option<DcNetDataLocal>,
    pub task: DcNetDataTaskStatus,
}

#[derive(Debug, Clone, Copy)]
pub struct DungeonRuntime {
    pub now: i32,
    pub zone_offset: i32,
    pub player_exp_per_stamina: i32,
}

pub struct DungeonSettlementOutcome {
    pub gameplay_id: i32,
    pub rewards: DcNetDataTakeRewardRes,
    pub remains: Vec<DcNetDataItem>,
    pub heat_update: Option<HeatInfoOutcome>,
    pub user_up_data: Option<DcNetDataAttrUpData>,
    pub mission_updates: Vec<DcNetDataMissonInfo>,
    pub achievement_updates: Vec<DcNetDataAchi>,
    pub archive_updates: Vec<DcNetDataArchiveInfo>,
    pub daily_updates: Vec<DcNetDataTaskCycle>,
    pub seven_day_updates: Vec<DcNetDataAct7Day>,
    pub battle_pass_updates: Vec<DcNetDataBattlePassTask>,
}

pub struct DungeonSweepOutcome {
    pub infos: Vec<DcNetDataDungeonSweepInfo>,
    pub remains: Vec<DcNetDataItem>,
    pub heat_update: Option<HeatInfoOutcome>,
    pub archive_updates: Vec<DcNetDataArchiveInfo>,
}

pub struct DailyRewardOutcome {
    pub rewards: DcNetDataTakeRewardRes,
    pub level_up_data: DcNetDataAttrUpData,
    pub archive_updates: Vec<DcNetDataArchiveInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckinState {
    pub aid: i32,
    pub check_days: i32,
    pub last_check_day: i32,
    pub claimed_days: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MailState {
    pub mail: DcNetDataEmail,
    pub expires_at: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectedGoldCoin {
    pub city_id: i32,
    pub coin_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionResourceState {
    pub collection_id: String,
    pub remaining: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayingPortState {
    pub port_index_id: i32,
    pub port_info: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MallPurchaseState {
    pub goods_id: i32,
    pub bought: i32,
    pub period: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RechargePurchaseState {
    pub recharge_id: i32,
    pub purchase_count: i32,
    pub last_bought_at: i32,
    pub expires_at: i32,
}

pub struct RechargePurchaseOutcome {
    pub reward: DcNetDataTakeRewardRes,
    pub purchase_count: i32,
}

pub struct MallPurchaseOutcome {
    pub reward: DcNetDataTakeRewardRes,
    pub remains: Vec<DcNetDataItem>,
    pub goods: DcNetDataMallGoods,
}

pub struct Player {
    pub uid: i64,
    pub username: String,
    pub nickname: String,
    pub level: i32,
    pub gameplay_id: i32,
    pub created_at: i32,
    pub last_login_time: i32,
    pub cur_form: i32,
    pub profile_avatar: i32,
    pub profile_card: i32,
    pub profile_title: i32,
    pub profile_frame: i32,
    pub banner_girl: i32,
    pub traced_task_group: i32,
    pub heat_updated_at: i32,
    pub items: Vec<DcNetDataItem>,
    pub item_acquired: HashMap<i32, i32>,
    pub item_spent: HashMap<i32, i32>,
    pub roles: Vec<DcNetDataRolesRolesDetail>,
    pub formations: Vec<DcNetDataFormation>,
    pub missions: Vec<DcNetDataMissonInfo>,
    pub tasks: Vec<DcNetDataTaskStatus>,
    pub ports: Vec<DcNetDataPort>,
    pub completed_dungeons: Vec<i32>,
    pub dungeon_clears: HashMap<i32, i32>,
    pub equips: Vec<DcNetDataEquip>,
    pub equipment_word_plans: HashMap<i64, Vec<i32>>,
    pub equipment_groups: Vec<DcNetDataEquipsGroup>,
    pub team_cores: Vec<DcNetDataTeamCore>,
    pub team_equips: Vec<DcNetDataTeamEquip>,
    pub skillstones: Vec<DcNetDataSkillStone>,
    pub partners: Vec<DcNetDataPartner>,
    pub feats: Vec<DcNetDataFeat>,
    pub skins: Vec<i32>,
    pub monster_manuals: Vec<DcNetDataMonsterManual4Port>,
    pub interact_objs: Vec<DcNetDataInteractObj>,
    pub challenges: Vec<DcNetDataChallenge>,
    pub albums: Vec<DcNetDataAlbums>,
    pub collections: Vec<DcNetDataCollection>,
    pub collection_suit_rewards: Vec<DcNetDataCollectionSuitReward>,
    pub collection_places: Vec<DcNetDataCollectionPlaceInfo>,
    pub tp_map: BTreeMap<String, String>,
    pub role_attrs: Vec<DcNetDataRoleAttrInfo>,
    pub playing_port: Option<PlayingPortState>,
    pub version_task_claims: Vec<i32>,
    pub dense_fogs: Vec<i32>,
    pub profile_frames: Vec<DcNetDataProfileFrame>,
    pub profile_avatars: Vec<DcNetDataProfileAvatar>,
    pub profile_cards: Vec<DcNetDataProfileCard>,
    pub profile_titles: Vec<DcNetDataProfileTitle>,
    pub city_guides: Vec<String>,
    pub user_guides: Vec<String>,
    pub region: i32,
    pub locals: Vec<DcNetDataLocal>,
    pub favors: Vec<DcNetDataFavor>,
    pub favor_day: i32,
    pub favor_touches: i32,
    pub sms: Vec<DcNetDataSms>,
    pub daily_day: i32,
    pub daily_activity: i32,
    pub daily_reward_progress: i32,
    pub heat_exchange_day: i32,
    pub heat_exchange_count: i32,
    pub daily_tasks: Vec<DcNetDataTaskCycle>,
    pub gold_coins: Vec<CollectedGoldCoin>,
    pub collection_resources: Vec<CollectionResourceState>,
    pub wild_monster_day: i32,
    pub defeated_monster_points: Vec<String>,
    pub region_coin_daily: HashMap<i32, i32>,
    pub region_ticket_day: i32,
    pub region_levels: HashMap<i32, i32>,
    pub reward_boxes: Vec<DcNetDataTBoxInfo>,
    pub achievements: Vec<DcNetDataAchi>,
    pub activity_7day_claims: Vec<DcNetDataAct7Day>,
    pub activity_7day_point_claims: Vec<i32>,
    pub activity_level_rewards: Vec<i32>,
    pub activity_limited_level_rewards: Vec<i32>,
    pub activity_challenge_point_claims: Vec<i32>,
    pub version_challenges: Vec<protocol::pbcommon::DcNetDataVersionChallengeInfo>,
    pub more_team_challenges: Vec<MoreTeamChallengeState>,
    pub gachas: Vec<GachaState>,
    pub gacha_logs: Vec<DcNetDataGachaLog>,
    pub checkins: Vec<CheckinState>,
    pub checked_red_dots: Vec<String>,
    pub mails: Vec<MailState>,
    pub battle_pass: BattlePassState,
    pub mall_purchases: Vec<MallPurchaseState>,
    pub recharge_purchases: Vec<RechargePurchaseState>,
    pub boss_rush: BossRushState,
    pub activity_boss: ActivityBossState,
    pub rift: RiftState,
    pub rouge_progress: HashMap<i32, i32>,
    pub rouge_technology: Vec<i32>,
    pub rouge_score: i32,
    pub rouge_score_point_at: i32,
    pub rouge_run: Option<DcNetDataRouge>,
    pub ship_tags: Vec<i32>,
    pub archive_unlocks: HashMap<i32, i32>,
}
