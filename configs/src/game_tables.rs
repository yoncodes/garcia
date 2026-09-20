use std::{collections::HashSet, path::Path};

use anyhow::{Context, Result};

use crate::{
    GroupedTable, Table, TableFile,
    table::load_json,
    tables::{
        Achievement, AchievementChain, AchievementGoal, Activity, ActivityBoss,
        ActivityBossDailyMilestone, ActivityBossMilestone, ActivityBossPort, ActivityChallenge,
        ActivityChallengeMilestone, ActivityLevelUp, Album, Archive, BattlePass, BattlePassReward,
        BattlePassTask, BossRush, BossRushRankingReward, BossRushReward, BreakableObject,
        CharacterFavor, CharacterFavorParameters, CharacterMessageGroup, CheckIn, CityChallenge,
        CityEvent, CityMap, CityRegion, CityStage, Collection, CollectionResource, CollectionSuit,
        CultivationConstants, DailyActivityMilestone, DailyTaskDefinition, Dungeon, DungeonPort,
        DungeonPortGroup, EquipmentDefinition, EquipmentLevel, EquipmentParameters, EventBlessing,
        EventDefinition, EventNpc, EventNpcChoice, EventNpcChoiceEffect, EventTrade,
        ExtraEquipmentWord, FeatureUnlock, FreeShopGoods, GachaDrawType, GachaPool, GachaReward,
        GachaWeight, GoldCoin, InteractLimit, ItemConversion, ItemDefinition, ItemSynthesisRecipe,
        MaidDefinition, MaidLevel, MaidRank, MaidSkin, MaidTalent, MaidTalentLevel,
        MainCityMonsterGroup, MainCityMonsterPoint, MainEquipmentWord, MainStoryPort, Mall,
        MallGoods, MallGoodsGroup, Mission, MonsterDefinition, MoreTeamChallenge,
        MoreTeamChallengeGroup, MoreTeamChallengePort, MoreTeamChallengeStar, PartnerAscension,
        PartnerDefinition, PartnerLevel, PartnerParameters, PartnerResonanceCost, PartnerSkill,
        PlayerLevel, ProfileAvatar, ProfileCosmetic, Recharge, RefreshBox, RegionReputationLevel,
        RewardBox, RewardBundle, Rift, RiftBuff, RiftPort, RiftReward, RogueBuff, RogueItem,
        RogueMode, RogueNode, RogueNodeGroup, RoguePort, RogueRules, RogueTechnology,
        RogueWeeklyReward, SevenDayActivity, SevenDayActivityMilestone, Shop, SkillStone, Task,
        TaskRegion, TeamCore, TeamCoreRecipe, TeamEquipmentDefinition, TeamEquipmentLevel,
        TeamEquipmentParameters, TeleportationAnchor, TreasureLocation, Trial, Tutorial,
        VersionActivity, VersionActivityTask, VersionActivityTaskGroup, VersionChallenge,
        VersionChallengePort, VersionChallengeReward, WorldLevel,
    },
};

#[derive(Debug)]
pub struct GameTables {
    pub cultivation_constants: CultivationConstants,
    pub maids: Table<i32, MaidDefinition>,
    pub maid_skins: Table<i32, MaidSkin>,
    pub maid_talents_by_maid: GroupedTable<i32, MaidTalent>,
    pub maid_talent_levels: Vec<MaidTalentLevel>,
    pub character_favor_by_character: GroupedTable<i32, CharacterFavor>,
    pub character_favor_parameters: Table<i32, CharacterFavorParameters>,
    pub character_message_groups: Table<i32, CharacterMessageGroup>,
    pub missions: Table<i32, Mission>,
    pub tasks: Table<i32, Task>,
    pub items: Table<i32, ItemDefinition>,
    pub item_conversions: Table<i32, ItemConversion>,
    pub item_synthesis_recipes: Table<i32, ItemSynthesisRecipe>,
    pub profile_avatars: Table<i32, ProfileAvatar>,
    pub profile_frames: Table<i32, ProfileCosmetic>,
    pub profile_cards: Table<i32, ProfileCosmetic>,
    pub profile_titles: Table<i32, ProfileCosmetic>,
    pub city_events: Table<String, CityEvent>,
    pub city_guides: Table<String, CityEvent>,
    pub interaction_limits: Table<String, InteractLimit>,
    pub teleportation_anchors: Table<String, TeleportationAnchor>,
    pub city_challenges: Table<String, CityChallenge>,
    pub feature_unlocks: Table<i32, FeatureUnlock>,
    pub albums: Table<i32, Album>,
    pub achievements: Table<i32, Achievement>,
    pub achievement_chains: Table<i32, AchievementChain>,
    pub achievement_goals: Table<i32, AchievementGoal>,
    pub seven_day_activities: Table<i32, SevenDayActivity>,
    pub seven_day_activity_milestones: Table<i32, SevenDayActivityMilestone>,
    pub activities: Table<i32, Activity>,
    pub activity_level_rewards: Table<i32, ActivityLevelUp>,
    pub limited_level_rewards: Table<i32, ActivityLevelUp>,
    pub activity_bosses: Table<i32, ActivityBoss>,
    pub activity_boss_points: Vec<ActivityBossMilestone>,
    pub activity_boss_daily_points: Vec<ActivityBossDailyMilestone>,
    pub activity_boss_ports: Table<i32, ActivityBossPort>,
    pub activity_challenges: Table<i32, ActivityChallenge>,
    pub activity_challenge_milestones: Table<i32, ActivityChallengeMilestone>,
    pub version_challenges: Table<i32, VersionChallenge>,
    pub version_challenge_rewards: Vec<VersionChallengeReward>,
    pub version_challenge_ports: Table<i32, VersionChallengePort>,
    pub more_team_challenges: Table<i32, MoreTeamChallenge>,
    pub more_team_challenge_groups: Table<i32, MoreTeamChallengeGroup>,
    pub more_team_challenge_stars: Vec<MoreTeamChallengeStar>,
    pub more_team_challenge_ports: Table<i32, MoreTeamChallengePort>,
    pub battle_passes: Table<i32, BattlePass>,
    pub battle_pass_tasks: Table<i32, BattlePassTask>,
    pub battle_pass_rewards: Vec<BattlePassReward>,
    pub boss_rushes: Table<i32, BossRush>,
    pub boss_rush_rewards: Vec<BossRushReward>,
    pub boss_rush_ranking_rewards: Vec<BossRushRankingReward>,
    pub check_ins_by_activity: GroupedTable<i32, CheckIn>,
    pub gacha_pools: Table<i32, GachaPool>,
    pub gacha_draw_types: Table<i32, GachaDrawType>,
    pub gacha_rewards: Vec<GachaReward>,
    pub gacha_weights: Table<i32, GachaWeight>,
    pub mall_goods_groups: Vec<MallGoodsGroup>,
    pub mall_goods: Vec<MallGoods>,
    pub malls: Table<i32, Mall>,
    pub shops: Table<i32, Shop>,
    pub free_shop_goods: Vec<FreeShopGoods>,
    pub city_maps: Table<i32, CityMap>,
    pub archives: Table<i32, Archive>,
    pub city_regions: Table<i32, CityRegion>,
    pub region_reputation_levels: Vec<RegionReputationLevel>,
    pub region_tasks_by_region: GroupedTable<i32, TaskRegion>,
    pub trials: Table<i32, Trial>,
    pub tutorials: Table<i32, Tutorial>,
    pub city_stages: Table<i32, CityStage>,
    pub recharges: Table<i32, Recharge>,
    pub partners: Table<i32, PartnerDefinition>,
    pub partner_levels_by_quality: GroupedTable<i32, PartnerLevel>,
    pub partner_parameters: Table<i32, PartnerParameters>,
    pub partner_ascensions_by_group: GroupedTable<i32, PartnerAscension>,
    pub partner_resonance_costs: Table<i32, PartnerResonanceCost>,
    pub partner_skills_by_group: GroupedTable<i32, PartnerSkill>,
    pub equipment: Table<i32, EquipmentDefinition>,
    pub equipment_levels: Table<i32, EquipmentLevel>,
    pub equipment_parameters: Table<i32, EquipmentParameters>,
    pub extra_equipment_words: Table<i32, ExtraEquipmentWord>,
    pub extra_equipment_words_by_slot: GroupedTable<i32, ExtraEquipmentWord>,
    pub main_equipment_words_by_group: GroupedTable<i32, MainEquipmentWord>,
    pub collections: Table<i32, Collection>,
    pub collection_suits: Table<i32, CollectionSuit>,
    pub main_story_ports: Table<i32, MainStoryPort>,
    pub dungeon_ports: Table<i32, DungeonPort>,
    pub dungeon_port_groups: Table<i32, DungeonPortGroup>,
    pub dungeons: Table<i32, Dungeon>,
    pub maid_levels: Table<i32, MaidLevel>,
    pub maid_ranks_by_maid: GroupedTable<i32, MaidRank>,
    pub player_levels: Table<i32, PlayerLevel>,
    pub team_cores: Table<i32, TeamCore>,
    pub team_core_recipes: Table<i32, TeamCoreRecipe>,
    pub team_equipment: Table<i32, TeamEquipmentDefinition>,
    pub team_equipment_levels: Table<i32, TeamEquipmentLevel>,
    pub team_equipment_parameters: Table<i32, TeamEquipmentParameters>,
    pub skill_stones: Table<i32, SkillStone>,
    pub daily_tasks: Table<i32, DailyTaskDefinition>,
    pub daily_activity_milestones: Table<i32, DailyActivityMilestone>,
    pub gold_coins: Table<String, GoldCoin>,
    pub collection_resources: Table<String, CollectionResource>,
    pub rewards: Table<i32, RewardBundle>,
    pub rifts: Table<i32, Rift>,
    pub rift_ports: Table<i32, RiftPort>,
    pub rift_buffs: Table<i32, RiftBuff>,
    pub rift_rewards: Vec<RiftReward>,
    pub rogue_modes: Table<i32, RogueMode>,
    pub rogue_buffs: Table<i32, RogueBuff>,
    pub rogue_nodes: Table<i32, RogueNode>,
    pub rogue_ports: Table<i32, RoguePort>,
    pub rogue_node_groups: Table<i32, RogueNodeGroup>,
    pub rogue_rules: RogueRules,
    pub event_blessings: Table<i32, EventBlessing>,
    pub event_trades: Table<i32, EventTrade>,
    pub rogue_items: Table<i32, RogueItem>,
    pub event_npcs: Table<i32, EventNpc>,
    pub event_npc_choices: Table<i32, EventNpcChoice>,
    pub event_npc_choice_effects: Table<i32, EventNpcChoiceEffect>,
    pub event_definitions: Table<i32, EventDefinition>,
    pub rogue_technology: Table<i32, RogueTechnology>,
    pub rogue_weekly_rewards: Vec<RogueWeeklyReward>,
    pub main_city_monster_groups: Table<String, MainCityMonsterGroup>,
    pub main_city_monster_points: Table<String, MainCityMonsterPoint>,
    pub monsters: Table<String, MonsterDefinition>,
    pub reward_boxes: Table<String, RewardBox>,
    pub refresh_boxes: Table<i32, RefreshBox>,
    pub treasure_locations: Table<i32, TreasureLocation>,
    pub breakable_objects: Table<String, BreakableObject>,
    pub world_levels: Vec<WorldLevel>,
    pub version_activities: Table<i32, VersionActivity>,
    pub version_activity_task_groups: Table<i32, VersionActivityTaskGroup>,
    pub version_activity_tasks: Table<i32, VersionActivityTask>,
}

impl GameTables {
    pub fn is_interaction_object(&self, id: &str) -> bool {
        self.interaction_limits.get(id).is_some()
            || self.city_events.get(id).is_some()
            || self.city_guides.get(id).is_some()
            || self.teleportation_anchors.get(id).is_some()
    }

    pub fn load(path: &Path) -> Result<Self> {
        macro_rules! rows {
            ($file:literal, $ty:ty) => {
                TableFile::<$ty>::load(&path.join($file))?.rows
            };
        }
        macro_rules! table {
            ($file:literal, $ty:ty, $key:ident) => {
                Table::new(TableFile::<$ty>::load(&path.join($file))?, $file, |row| {
                    row.$key
                })?
            };
        }
        macro_rules! string_table {
            ($file:literal, $ty:ty, $key:ident) => {
                Table::new(TableFile::<$ty>::load(&path.join($file))?, $file, |row| {
                    row.$key.clone()
                })?
            };
        }
        macro_rules! grouped {
            ($file:literal, $ty:ty, $key:ident) => {
                GroupedTable::new(TableFile::<$ty>::load(&path.join($file))?, |row| row.$key)
            };
        }

        let cultivation_constants = load_json(&path.join("constdefine_cultivate.json"))?;
        let extra_equipment_word_rows = rows!("equipword_extra.json", ExtraEquipmentWord);
        let extra_equipment_words = Table::new(
            TableFile {
                rows: extra_equipment_word_rows.clone(),
            },
            "equipword_extra.json",
            |row| row.id,
        )?;
        let extra_equipment_words_by_slot = GroupedTable::new(
            TableFile {
                rows: extra_equipment_word_rows,
            },
            |row| row.slot,
        );

        Ok(Self {
            cultivation_constants,
            maids: table!("maidconfig.json", MaidDefinition, id),
            maid_skins: table!("maid_skin.json", MaidSkin, id),
            maid_talents_by_maid: grouped!("maid_talent.json", MaidTalent, maid_id),
            maid_talent_levels: rows!("maid_talent_lv.json", MaidTalentLevel),
            character_favor_by_character: grouped!(
                "characterfavor.json",
                CharacterFavor,
                character_id
            ),
            character_favor_parameters: table!(
                "characterfavorparam.json",
                CharacterFavorParameters,
                id
            ),
            character_message_groups: table!(
                "charactermessageggroup.json",
                CharacterMessageGroup,
                id
            ),
            missions: table!("misson.json", Mission, id),
            tasks: table!("task.json", Task, id),
            items: table!("itemconfig.json", ItemDefinition, id),
            item_conversions: table!("item_convert.json", ItemConversion, id),
            item_synthesis_recipes: table!("item_synthesis.json", ItemSynthesisRecipe, id),
            profile_avatars: table!("profile_avatar.json", ProfileAvatar, id),
            profile_frames: table!("profile_frame.json", ProfileCosmetic, id),
            profile_cards: table!("profile_card.json", ProfileCosmetic, id),
            profile_titles: table!("profile_title.json", ProfileCosmetic, id),
            city_events: string_table!("city_event.json", CityEvent, interact_id),
            city_guides: string_table!("city_guide.json", CityEvent, interact_id),
            interaction_limits: string_table!("interact_limit.json", InteractLimit, interact_id),
            teleportation_anchors: string_table!(
                "teleportationanchor.json",
                TeleportationAnchor,
                id
            ),
            city_challenges: string_table!("city_challenge.json", CityChallenge, city_challenge_id),
            feature_unlocks: table!("openconfig.json", FeatureUnlock, id),
            albums: table!("albumconfig.json", Album, id),
            achievements: table!("achievement.json", Achievement, id),
            achievement_chains: table!("achievement_chain.json", AchievementChain, id),
            achievement_goals: table!("achievement_goal.json", AchievementGoal, id),
            seven_day_activities: table!("activity_7day.json", SevenDayActivity, id),
            seven_day_activity_milestones: table!(
                "activity_7day_point.json",
                SevenDayActivityMilestone,
                id
            ),
            activities: table!("activity.json", Activity, id),
            activity_level_rewards: table!("activity_level_up.json", ActivityLevelUp, id),
            limited_level_rewards: table!("activity_level_up_limited.json", ActivityLevelUp, id),
            activity_bosses: table!("activity_boss.json", ActivityBoss, id),
            activity_boss_points: rows!("activity_boss_point.json", ActivityBossMilestone),
            activity_boss_daily_points: rows!(
                "activity_boss_daily_point.json",
                ActivityBossDailyMilestone
            ),
            activity_boss_ports: table!("gameplay_port_activity_boss.json", ActivityBossPort, id),
            activity_challenges: table!("activity_challenge.json", ActivityChallenge, id),
            activity_challenge_milestones: table!(
                "activity_challenge_point.json",
                ActivityChallengeMilestone,
                id
            ),
            version_challenges: table!("version_activity_challenge.json", VersionChallenge, id),
            version_challenge_rewards: rows!(
                "version_activity_challenge_reward.json",
                VersionChallengeReward
            ),
            version_challenge_ports: table!(
                "gameplay_port_version_activity_challenge.json",
                VersionChallengePort,
                id
            ),
            more_team_challenges: table!("more_team_challenge.json", MoreTeamChallenge, id),
            more_team_challenge_groups: table!(
                "more_team_challenge_group.json",
                MoreTeamChallengeGroup,
                id
            ),
            more_team_challenge_stars: rows!(
                "more_team_challenge_star.json",
                MoreTeamChallengeStar
            ),
            more_team_challenge_ports: table!(
                "gameplay_port_more_team_challenge.json",
                MoreTeamChallengePort,
                id
            ),
            battle_passes: table!("battle_pass.json", BattlePass, id),
            battle_pass_tasks: table!("battle_pass_task.json", BattlePassTask, id),
            battle_pass_rewards: rows!("battle_pass_reward.json", BattlePassReward),
            boss_rushes: table!("bossrush.json", BossRush, id),
            boss_rush_rewards: rows!("bossrush_point_reward.json", BossRushReward),
            boss_rush_ranking_rewards: rows!("bossrush_ranking_reward.json", BossRushRankingReward),
            check_ins_by_activity: grouped!("check_in.json", CheckIn, activity_id),
            gacha_pools: table!("drawconfig.json", GachaPool, id),
            gacha_draw_types: table!("draw_type_param.json", GachaDrawType, id),
            gacha_rewards: rows!("drawrewardconfig.json", GachaReward),
            gacha_weights: table!("gacha_weight.json", GachaWeight, gacha_id),
            mall_goods_groups: rows!("mall_goods_group.json", MallGoodsGroup),
            mall_goods: rows!("mall_goods.json", MallGoods),
            malls: table!("mall.json", Mall, id),
            shops: table!("shop.json", Shop, id),
            free_shop_goods: rows!("free_shop_goods.json", FreeShopGoods),
            city_maps: table!("city_map.json", CityMap, id),
            archives: table!("archive.json", Archive, id),
            city_regions: table!("city_region.json", CityRegion, id),
            region_reputation_levels: rows!("region_reputation_lv.json", RegionReputationLevel),
            region_tasks_by_region: grouped!("task_region.json", TaskRegion, region_id),
            trials: table!("trial.json", Trial, id),
            tutorials: table!("tutorial.json", Tutorial, id),
            city_stages: table!("city_stage.json", CityStage, city_id),
            recharges: table!("recharge.json", Recharge, id),
            partners: table!("partnerconfig.json", PartnerDefinition, id),
            partner_levels_by_quality: grouped!("partnerlevel.json", PartnerLevel, quality),
            partner_parameters: table!("partnerparam.json", PartnerParameters, quality),
            partner_ascensions_by_group: grouped!("partnerbreak.json", PartnerAscension, group_id),
            partner_resonance_costs: table!(
                "partnerresoncost.json",
                PartnerResonanceCost,
                resonance_level
            ),
            partner_skills_by_group: grouped!("partnerskill.json", PartnerSkill, skill_group),
            equipment: table!("equipconfig.json", EquipmentDefinition, id),
            equipment_levels: table!("equiplvconfig.json", EquipmentLevel, level),
            equipment_parameters: table!("equipparaconfig.json", EquipmentParameters, quality),
            extra_equipment_words,
            extra_equipment_words_by_slot,
            main_equipment_words_by_group: grouped!(
                "equipword_main.json",
                MainEquipmentWord,
                group_id
            ),
            collections: table!("collection.json", Collection, id),
            collection_suits: table!("collection_suit.json", CollectionSuit, id),
            main_story_ports: table!("gameplay_port_mainstory.json", MainStoryPort, id),
            dungeon_ports: table!("gameplay_port_dungeon.json", DungeonPort, id),
            dungeon_port_groups: table!(
                "gameplay_port_dungeon_group.json",
                DungeonPortGroup,
                port_group_id
            ),
            dungeons: table!("dungeon.json", Dungeon, dungeon_id),
            maid_levels: table!("maidlevelconfig.json", MaidLevel, level),
            maid_ranks_by_maid: grouped!("maidrankconfig.json", MaidRank, maid_id),
            player_levels: table!("playerlv.json", PlayerLevel, level),
            team_cores: table!("team_core.json", TeamCore, id),
            team_core_recipes: table!("team_core_synthesis.json", TeamCoreRecipe, id),
            team_equipment: table!("team_equip.json", TeamEquipmentDefinition, id),
            team_equipment_levels: table!("team_equip_lv.json", TeamEquipmentLevel, level),
            team_equipment_parameters: table!("team_equip_param.json", TeamEquipmentParameters, id),
            skill_stones: table!("skillstone.json", SkillStone, id),
            daily_tasks: table!("task_daily.json", DailyTaskDefinition, id),
            daily_activity_milestones: table!("task_daily_active.json", DailyActivityMilestone, id),
            gold_coins: string_table!("gold_coin.json", GoldCoin, goldcoin_id),
            collection_resources: string_table!(
                "collection_res.json",
                CollectionResource,
                collection_id
            ),
            rewards: table!("rewardconfig.json", RewardBundle, id),
            rifts: table!("rift.json", Rift, id),
            rift_ports: table!("gameplay_port_rift.json", RiftPort, id),
            rift_buffs: table!("rift_buff.json", RiftBuff, id),
            rift_rewards: rows!("rift_reward.json", RiftReward),
            rogue_modes: table!("rouge.json", RogueMode, id),
            rogue_buffs: table!("rouge_buff.json", RogueBuff, id),
            rogue_nodes: table!("node.json", RogueNode, id),
            rogue_ports: table!("gameplay_port_rouge.json", RoguePort, id),
            rogue_node_groups: table!("node_group.json", RogueNodeGroup, id),
            rogue_rules: rows!("rouge_rules.json", RogueRules)
                .into_iter()
                .next()
                .context("rouge_rules.json is empty")?,
            event_blessings: table!("event_blessing.json", EventBlessing, id),
            event_trades: table!("event_trade.json", EventTrade, id),
            rogue_items: table!("rouge_item.json", RogueItem, id),
            event_npcs: table!("event_npc.json", EventNpc, id),
            event_npc_choices: table!("event_npc_choose.json", EventNpcChoice, id),
            event_npc_choice_effects: table!(
                "event_npc_choose_effect.json",
                EventNpcChoiceEffect,
                id
            ),
            event_definitions: table!("event_define.json", EventDefinition, id),
            rogue_technology: table!("rouge_technology_tree.json", RogueTechnology, id),
            rogue_weekly_rewards: rows!("rouge_week_reward.json", RogueWeeklyReward),
            main_city_monster_groups: string_table!(
                "maincity_monstergroup.json",
                MainCityMonsterGroup,
                group_id
            ),
            main_city_monster_points: string_table!(
                "maincity_monsterpoint.json",
                MainCityMonsterPoint,
                id
            ),
            monsters: string_table!("monsterconfig.json", MonsterDefinition, id),
            reward_boxes: string_table!("tbox_reward.json", RewardBox, id),
            refresh_boxes: table!("tbox_process.json", RefreshBox, id),
            treasure_locations: table!("treasure_weight.json", TreasureLocation, id),
            breakable_objects: string_table!("bkobj_define.json", BreakableObject, id),
            world_levels: rows!("world_level.json", WorldLevel),
            version_activities: table!("version_activity.json", VersionActivity, id),
            version_activity_task_groups: table!(
                "version_activity_task_group.json",
                VersionActivityTaskGroup,
                id
            ),
            version_activity_tasks: table!("version_activity_task.json", VersionActivityTask, id),
        })
    }

    pub fn default_main_maid(&self) -> i32 {
        self.cultivation_constants.default_main_maid_sex[0]
    }

    pub fn profile_avatar_for_maid(&self, maid_id: i32) -> Option<&ProfileAvatar> {
        self.profile_avatars
            .rows
            .iter()
            .find(|avatar| avatar.maid_id == maid_id)
    }

    pub fn profile_card_for_maid(&self, maid_id: i32) -> Option<&ProfileCosmetic> {
        self.profile_cards
            .rows
            .iter()
            .find(|card| card.maid_id == Some(maid_id))
    }

    pub fn initial_talents(&self, maid_id: i32) -> impl Iterator<Item = (i32, i32)> + '_ {
        let quality = self.maids.get(maid_id).map_or(0, |maid| maid.quality);
        self.maid_talents_by_maid
            .get(maid_id)
            .into_iter()
            .flatten()
            .map(move |talent| {
                let level = self
                    .maid_talent_levels
                    .iter()
                    .find(|row| {
                        row.quality == quality
                            && row.position == talent.position
                            && row.unlock_limit.is_none()
                    })
                    .map_or(0, |row| row.level);
                (talent.position, level)
            })
    }

    pub fn is_playable_maid(&self, maid_id: i32) -> bool {
        self.maid_talents_by_maid
            .get(maid_id)
            .is_some_and(|mut talents| talents.next().is_some())
    }

    pub fn initial_task_ids(&self) -> impl Iterator<Item = i32> + '_ {
        let referenced: HashSet<i32> = self
            .tasks
            .rows
            .iter()
            .flat_map(|task| &task.next)
            .filter_map(|next| next.parse().ok())
            .collect();
        self.tasks.rows.iter().filter_map(move |task| {
            (task.task_type == 1 && task.sort == 1 && !referenced.contains(&task.id))
                .then_some(task.id)
        })
    }

    pub fn initial_location(&self) -> Option<(i32, String)> {
        let task = self
            .initial_task_ids()
            .filter_map(|id| self.tasks.get(id))
            .find(|task| {
                task.position
                    .as_ref()
                    .is_some_and(|position| position.key == 4)
            })?;
        let location = task.position.as_ref()?.value.clone();
        let map_id = self
            .city_maps
            .rows
            .iter()
            .find(|map| map.city_id == task.city_id)?
            .id;
        Some((map_id, location))
    }
}
