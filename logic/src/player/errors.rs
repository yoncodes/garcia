#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum MissionClaimError {
    #[error("unknown mission {0}")]
    Unknown(i32),
    #[error("mission {0} is not active")]
    Inactive(i32),
    #[error("mission {0} is incomplete")]
    Incomplete(i32),
    #[error("mission {0} was already claimed")]
    AlreadyClaimed(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum NamingError {
    #[error("unknown naming type {0}")]
    InvalidType(i32),
    #[error("unknown gender {0}")]
    InvalidGender(i32),
    #[error("name must be 1-12 letters, digits, or CJK characters")]
    InvalidName,
    #[error("rename cost is not configured")]
    MissingCost,
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientCost {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ProfileSelectionError {
    #[error("profile type {profile_type} does not contain id {id}")]
    Locked { profile_type: i32, id: i32 },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("album {0} is not unlocked")]
pub struct AlbumSelectionError(pub i32);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum FormationError {
    #[error("formation configuration is invalid")]
    InvalidConfig,
    #[error("the custom formation limit has been reached")]
    LimitReached,
    #[error("formation {0} does not exist")]
    Unknown(i32),
    #[error("fixed formation {0} cannot be deleted")]
    Fixed(i32),
    #[error("current formation {0} cannot be deleted")]
    Current(i32),
    #[error("formation {formation_id} has {members} members; maximum is {maximum}")]
    TooManyMembers {
        formation_id: i32,
        members: usize,
        maximum: usize,
    },
    #[error("role {0} is not owned")]
    RoleNotOwned(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InteractionError {
    #[error("unknown interaction object {0}")]
    Unknown(String),
    #[error("interaction {object_id} has unsupported reward {reward_id}")]
    UnsupportedReward { object_id: String, reward_id: i32 },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("unknown city guide {0}")]
pub struct CityGuideError(pub String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CityChallengeError {
    #[error("unknown city challenge {0}")]
    Unknown(String),
    #[error("city challenge {id} reported invalid star count {stars}")]
    InvalidStars { id: String, stars: i32 },
    #[error("city challenge {id} is missing reward preview tier {tier}")]
    MissingRewardPreview { id: String, tier: i32 },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum HeatExchangeError {
    #[error("heat exchange amount must be 1, got {0}")]
    InvalidAmount(i32),
    #[error("the daily heat exchange limit has been reached")]
    DailyLimit,
    #[error("heat is too high to exchange")]
    HeatFull,
    #[error("heat exchange configuration is missing")]
    MissingConfig,
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientCost {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CurrencyExchangeError {
    #[error("exchange amount must be positive")]
    InvalidAmount,
    #[error("diamond-to-stamp exchange configuration is invalid")]
    MissingConfig,
    #[error("not enough diamonds: need {needed}, have {available}")]
    InsufficientDiamonds { needed: i32, available: i32 },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RougeTechnologyError {
    #[error("unknown Rogue technology {0}")]
    Unknown(i32),
    #[error("Rogue technology {0} is already unlocked")]
    AlreadyUnlocked(i32),
    #[error("Rogue technology {0} is locked")]
    Locked(i32),
    #[error("Rogue technology {0} has an invalid cost")]
    InvalidCost(i32),
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientCost {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RougeScoreRewardError {
    #[error("no Rogue score reward is available")]
    NothingToClaim,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RougeRunError {
    #[error("a Rogue run is already active")]
    AlreadyActive,
    #[error("no Rogue run is active")]
    NotActive,
    #[error("Rogue stage {0} is not available")]
    Locked(i32),
    #[error("Rogue teams must contain one to three roles")]
    InvalidRoleCount,
    #[error("role {0} is not owned")]
    UnknownRole(i32),
    #[error("role {0} appears more than once")]
    DuplicateRole(i32),
    #[error("team equipment {0} is not owned")]
    UnknownTeamEquip(i64),
    #[error("invalid Rogue buff tag {0}")]
    InvalidBuffTag(i32),
    #[error("invalid Rogue status {0}")]
    InvalidStatus(i32),
    #[error("invalid Rogue gate position {0}")]
    InvalidEventPosition(i32),
    #[error("the active Rogue event does not allow this action")]
    InvalidEvent,
    #[error("Rogue buff {0} is not currently offered")]
    InvalidBuff(i32),
    #[error("unknown Rogue item {0}")]
    UnknownRougeItem(i32),
    #[error("invalid Rogue shop goods type {0}")]
    InvalidRougeGoodsType(i32),
    #[error("Rogue shop item {0} does not have enough stock")]
    InsufficientRougeStock(i32),
    #[error("not enough Rogue coin: need {needed}, have {available}")]
    InsufficientRougeCoin { needed: i32, available: i32 },
    #[error("the Rogue boss box was already opened")]
    BossBoxAlreadyOpened,
    #[error("the Rogue boss box cost is unavailable")]
    InsufficientBossBoxCost,
    #[error("Rogue table data for stage {0} is incomplete")]
    InvalidConfig(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ActivityBossError {
    #[error("unknown activity boss {0}")]
    Unknown(i32),
    #[error("activity boss {0} is not active")]
    Inactive(i32),
    #[error("activity boss damage cannot be negative")]
    NegativeDamage,
    #[error("unknown role {0}")]
    UnknownRole(i32),
    #[error("unknown activity-boss reward {0}")]
    UnknownReward(i32),
    #[error("activity-boss reward {0} has not been reached")]
    RewardNotReached(i32),
    #[error("activity-boss reward {0} was already claimed")]
    RewardAlreadyClaimed(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ActivityChallengeError {
    #[error("unknown activity challenge event {0}")]
    UnknownActivity(i32),
    #[error("activity challenge event {0} is not active")]
    Inactive(i32),
    #[error("unknown activity challenge {0}")]
    UnknownChallenge(i32),
    #[error("activity challenge {0} is disabled")]
    Disabled(i32),
    #[error("activity challenge {0} is incomplete")]
    Incomplete(i32),
    #[error("activity challenge {0} was already claimed")]
    AlreadyClaimed(i32),
    #[error("unknown activity challenge point reward {0}")]
    UnknownPointReward(i32),
    #[error("activity challenge point reward {0} has not been reached")]
    PointNotReached(i32),
    #[error("activity challenge point reward {0} was already claimed")]
    PointAlreadyClaimed(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum VersionChallengeError {
    #[error("the version challenge activity is not active")]
    Inactive,
    #[error("unknown version challenge {0}")]
    Unknown(i32),
    #[error("version challenge {0} is locked")]
    Locked(i32),
    #[error("version challenge {id} requires player level {required}")]
    LevelLocked { id: i32, required: i32 },
    #[error("version challenge {0} has no gameplay port")]
    MissingPort(i32),
    #[error("version challenge {0} settlement is missing valid metrics")]
    InvalidResult(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum MoreTeamChallengeError {
    #[error("more-team challenge group {0} is closed or locked")]
    Closed(i32),
    #[error("unknown more-team challenge {0}")]
    Unknown(i32),
    #[error("more-team challenge {0} is locked")]
    Locked(i32),
    #[error("more-team challenge {id} requires player level {required}")]
    LevelLocked { id: i32, required: i32 },
    #[error("more-team challenge {0} has no gameplay port")]
    MissingPort(i32),
    #[error("more-team challenge {0} settlement has an invalid completion time")]
    InvalidTime(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TaskCompleteError {
    #[error("unknown task {0}")]
    Unknown(i32),
    #[error("task {0} is not active")]
    Inactive(i32),
    #[error("task {0} is already complete")]
    AlreadyComplete(i32),
    #[error("task {task_id} has unsupported reward {reward_id}")]
    UnsupportedReward { task_id: i32, reward_id: i32 },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TaskGoalError {
    #[error("task group {0} is not active")]
    Inactive(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TeamCoreError {
    #[error("team-core configuration {0} does not exist")]
    MissingConfig(i32),
    #[error("unknown team-core synthesis {0}")]
    UnknownSynthesis(i32),
    #[error("team-core synthesis amount must be positive")]
    InvalidAmount,
    #[error("team-core synthesis {0} has invalid targets")]
    InvalidTargets(i32),
    #[error("team core instance {0} does not exist")]
    UnknownCore(i64),
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientCost {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TeamEquipError {
    #[error("team equipment instance {0} does not exist")]
    UnknownEquip(i64),
    #[error("team core instance {0} does not exist")]
    UnknownCore(i64),
    #[error("formation {0} does not exist")]
    UnknownFormation(i32),
    #[error("team equipment configuration {0} does not exist")]
    MissingConfig(i32),
    #[error("core position {position} is invalid for team equipment {equip_id}")]
    InvalidCorePosition { equip_id: i64, position: i32 },
    #[error("team equipment instance id space is exhausted")]
    InstanceIdExhausted,
    #[error("team equipment {0} is already at its level cap")]
    LevelCap(i64),
    #[error("invalid team-equipment EXP item {0}")]
    InvalidMaterial(i32),
    #[error("team-equipment EXP item {item_id} used the wrong instance id")]
    WrongItemInstance { item_id: i32 },
    #[error("team-equipment EXP item {item_id} needs {needed}, only {available} available")]
    InsufficientMaterial {
        item_id: i32,
        needed: i32,
        available: i32,
    },
    #[error("team equipment {0} cannot be consumed while locked, equipped, or holding cores")]
    MaterialUnavailable(i64),
    #[error("team equipment {0} was selected as material more than once")]
    DuplicateMaterial(i64),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SkillStoneError {
    #[error("skill stone instance {0} does not exist")]
    UnknownStone(i64),
    #[error("role {0} does not exist")]
    UnknownRole(i32),
    #[error("skill stone configuration {0} does not exist")]
    MissingConfig(i32),
    #[error("skill stone {stone_id} belongs to role {expected}, not {actual}")]
    WrongRole {
        stone_id: i64,
        expected: i32,
        actual: i32,
    },
    #[error("skill stone instance id space is exhausted")]
    InstanceIdExhausted,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum FeatureError {
    #[error("feature {0} is not unlockable")]
    NotUnlockable(i32),
    #[error("feature {feature_id} uses unsupported requirement type {condition}")]
    UnsupportedRequirement { feature_id: i32, condition: i32 },
    #[error("feature requirement references unknown task {0}")]
    UnknownRequirementTask(i32),
    #[error("feature {feature_id} has invalid requirement value `{value}`")]
    InvalidRequirementValue { feature_id: i32, value: String },
    #[error("could not establish feature progression: {0}")]
    Progression(String),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ProgressionError {
    #[error("unknown player level {0}")]
    UnknownPlayerLevel(i32),
    #[error("player level can only increase (current {current}, requested {requested})")]
    PlayerLevelDecrease { current: i32, requested: i32 },
    #[error("unknown world level {0}")]
    UnknownWorldLevel(i32),
    #[error("world level can only increase (current {current}, requested {requested})")]
    WorldLevelDecrease { current: i32, requested: i32 },
    #[error("gameplay {0} is not a story stage")]
    UnknownStoryStage(i32),
    #[error("story stage {0} is not connected to the main task chain")]
    UnlinkedStoryStage(i32),
    #[error("task {0} does not exist")]
    UnknownTask(i32),
    #[error("task {task_id} has invalid condition {condition} value `{value}`")]
    InvalidCondition {
        task_id: i32,
        condition: i32,
        value: String,
    },
    #[error("task {task_id} uses unsupported completion condition {condition}")]
    UnsupportedCondition { task_id: i32, condition: i32 },
    #[error("task progression failed: {0}")]
    Task(String),
    #[error("story settlement failed: {0}")]
    Port(String),
    #[error("interaction progression failed: {0}")]
    Interaction(String),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CollectionError {
    #[error("unknown collection suit {0}")]
    UnknownSuit(i32),
    #[error("unknown reward step {step} for collection suit {suit_id}")]
    UnknownStep { suit_id: i32, step: i32 },
    #[error("collection suit {suit_id} step {step} requires {required} collections")]
    SuitIncomplete {
        suit_id: i32,
        step: i32,
        required: i32,
    },
    #[error("collection suit {suit_id} step {step} was already claimed")]
    SuitRewardClaimed { suit_id: i32, step: i32 },
    #[error("unknown collection {0}")]
    UnknownCollection(i32),
    #[error("collection {0} is not owned")]
    CollectionNotOwned(i32),
    #[error("collection {0} has no individual reward")]
    MissingReward(i32),
    #[error("collection {0} reward was already claimed")]
    CollectionRewardClaimed(i32),
    #[error("invalid collection display platform {0}")]
    InvalidPlatform(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RoleAttrError {
    #[error("role {0} is not owned")]
    RoleNotOwned(i32),
    #[error("role {0} appears more than once")]
    DuplicateRole(i32),
    #[error("role {0} has a negative saved attribute")]
    NegativeAttribute(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum VersionTaskError {
    #[error("the version activity is not active")]
    Inactive,
    #[error("unknown version task {0}")]
    Unknown(i32),
    #[error("version task {0} is incomplete")]
    Incomplete(i32),
    #[error("version task {0} was already claimed")]
    AlreadyClaimed(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CraftError {
    #[error("unknown synthesis formula {0}")]
    UnknownSynthesis(i32),
    #[error("unknown conversion formula {0}")]
    UnknownConversion(i32),
    #[error("invalid craft amount {0}")]
    InvalidAmount(i32),
    #[error("craft formula {0} is locked")]
    Locked(i32),
    #[error("craft target item {0} does not exist")]
    UnknownTarget(i32),
    #[error("conversion needs {expected} material slots, got {actual}")]
    WrongMaterialCount { expected: usize, actual: usize },
    #[error("item {0} is not a valid conversion material")]
    InvalidMaterial(i32),
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientMaterial {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InventoryError {
    #[error("unknown item {0}")]
    UnknownItem(i32),
    #[error("item {0} has no use effect")]
    MissingEffect(i32),
    #[error("invalid item amount {0}")]
    InvalidAmount(i32),
    #[error("item type {0} cannot be used")]
    UnsupportedType(i32),
    #[error("item {0} must use the selectable-package command")]
    SelectablePackage(i32),
    #[error("item {0} is not a selectable package")]
    NotSelectablePackage(i32),
    #[error("unknown reward {0}")]
    UnknownReward(i32),
    #[error("relic package reward {0} contains a non-relic entry")]
    InvalidRelicPackage(i32),
    #[error("reward {0} is not selectable from this package")]
    InvalidSelection(i32),
    #[error("the package selection is empty")]
    EmptySelection,
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientItem {
        item_id: i32,
        needed: i32,
        available: i32,
    },
    #[error("heat is already full")]
    HeatFull,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ShopPurchaseError {
    #[error("unknown shop goods {0}")]
    Unknown(i32),
    #[error("shop goods {0} is not active")]
    Inactive(i32),
    #[error("invalid shop purchase amount {0}")]
    InvalidAmount(i32),
    #[error("shop goods {id} has a purchase limit of {limit}")]
    BuyLimit { id: i32, limit: i32 },
    #[error("shop goods {goods_id} has unsupported reward {reward_id}")]
    UnsupportedReward { goods_id: i32, reward_id: i32 },
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientCost {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DisassemblyError {
    #[error("nothing was selected for disassembly")]
    Empty,
    #[error("entity {0} appears more than once")]
    Duplicate(i64),
    #[error("unknown disassembly entity {0}")]
    Unknown(i64),
    #[error("entity {user_id} is not item {item_id}")]
    ItemMismatch { user_id: i64, item_id: i32 },
    #[error("entity {0} is locked")]
    Locked(i64),
    #[error("entity {0} is equipped")]
    Equipped(i64),
    #[error("too many {kind} selected: maximum is {limit}")]
    TooMany { kind: &'static str, limit: i32 },
    #[error("missing disassembly configuration for item {0}")]
    MissingConfig(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TrialError {
    #[error("unknown trial {0}")]
    Unknown(i32),
    #[error("trial {0} is locked")]
    Locked(i32),
    #[error("the active formation is empty")]
    EmptyFormation,
    #[error("trial {0} is already complete")]
    AlreadyComplete(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PortError {
    #[error("unknown gameplay {0}")]
    Unknown(i32),
    #[error("unknown monster CRC {0}")]
    UnknownMonsterCrc(u32),
    #[error("settlement for gameplay {actual} received while {expected} is active")]
    NotActive { expected: i32, actual: i32 },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DungeonError {
    #[error("unknown dungeon gameplay {0}")]
    Unknown(i32),
    #[error("dungeon gameplay {0} is locked")]
    Locked(i32),
    #[error("dungeon gameplay {id} has type {expected}, request sent {actual}")]
    WrongType { id: i32, expected: i32, actual: i32 },
    #[error("settlement for dungeon {actual} received while {expected} is active")]
    NotActive { expected: i32, actual: i32 },
    #[error("dungeon multiplier must be between 1 and {maximum}, got {actual}")]
    InvalidMultiplier { maximum: i32, actual: i32 },
    #[error("dungeon sweep count must be positive")]
    InvalidSweepCount,
    #[error("dungeon gameplay {0} has not been completed")]
    NotCompleted(i32),
    #[error("dungeon gameplay {0} cannot be swept")]
    SweepDisabled(i32),
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientCost {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum FavorError {
    #[error("unknown character {0}")]
    UnknownCharacter(i32),
    #[error("character id {0} is outside the supported range")]
    InvalidCharacterId(i64),
    #[error("the daily favor touch limit has been reached")]
    DailyLimit,
    #[error("character {0} is already at maximum favor")]
    MaxLevel(i32),
    #[error("item {0} is not a favor material")]
    InvalidMaterial(i32),
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientItem {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SmsError {
    #[error("SMS group {0} is not unlocked")]
    Locked(i32),
    #[error("invalid SMS selection {0}")]
    InvalidSelection(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DailyTaskError {
    #[error("unknown daily task {0}")]
    Unknown(i32),
    #[error("daily task {0} is incomplete")]
    Incomplete(i32),
    #[error("daily task {0} activity was already taken")]
    AlreadyTaken(i32),
    #[error("no daily activity reward is available")]
    NoReward,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GoldCoinError {
    #[error("unknown gold coin {0}")]
    Unknown(String),
    #[error("gold coin {0} was already collected")]
    AlreadyCollected(String),
    #[error("gold coin reward item is missing from itemconfig")]
    MissingRewardItem,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CollectionResourceError {
    #[error("unknown collection resource {0}")]
    Unknown(String),
    #[error("collection resource {0} is depleted")]
    Depleted(String),
    #[error("collection resource {0} has no configured reward")]
    MissingReward(String),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum MonsterPointError {
    #[error("unknown monster point {0}")]
    Unknown(String),
    #[error("monster point {0} references an unknown group")]
    UnknownGroup(String),
    #[error("monster point {point_id} references unknown monster {monster_id}")]
    UnknownMonster {
        point_id: String,
        monster_id: String,
    },
    #[error("monster {monster_id} references unknown reward {reward_id}")]
    MissingMonsterReward { monster_id: String, reward_id: i32 },
    #[error("region coin item {0} is missing from itemconfig")]
    MissingRegionCoin(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RegionError {
    #[error("unknown region {0}")]
    UnknownRegion(i32),
    #[error("region {region_id} does not offer task group {group_id}")]
    UnknownTaskGroup { region_id: i32, group_id: i32 },
    #[error("region task group {0} is not in today's rotation")]
    NotOffered(i32),
    #[error("region task group {0} was already accepted today")]
    AlreadyAccepted(i32),
    #[error("the active region task limit has been reached")]
    ActiveLimit,
    #[error("not enough region ticket {item_id}: need {needed}, have {available}")]
    InsufficientTickets {
        item_id: i32,
        needed: i32,
        available: i32,
    },
    #[error("region task group {0} has no task")]
    EmptyTaskGroup(i32),
    #[error("the daily region ticket was already claimed")]
    DailyTicketClaimed,
    #[error("the region ticket inventory is full")]
    TicketFull,
    #[error("region ticket configuration is missing")]
    MissingTicketConfig,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RewardBoxError {
    #[error("unknown reward box {0}")]
    Unknown(String),
    #[error("reward box {0} was already claimed")]
    AlreadyClaimed(String),
    #[error("reward box {0} has no configured reward")]
    MissingReward(String),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AchievementError {
    #[error("unknown achievement {0}")]
    Unknown(i32),
    #[error("achievement {0} is incomplete")]
    Incomplete(i32),
    #[error("achievement {0} was already claimed")]
    AlreadyClaimed(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Activity7DayError {
    #[error("unknown seven-day activity task {0}")]
    Unknown(i32),
    #[error("seven-day activity task {0} is locked")]
    Locked(i32),
    #[error("seven-day activity task {0} is incomplete")]
    Incomplete(i32),
    #[error("seven-day activity reward {0} was already claimed")]
    AlreadyClaimed(i32),
    #[error("seven-day point reward {0} has an invalid requirement")]
    InvalidPoint(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GachaError {
    #[error("unknown gacha pool {0}")]
    Unknown(i32),
    #[error("gacha pool {0} is not active")]
    Inactive(i32),
    #[error("gacha count must be 1 or 10, got {0}")]
    InvalidCount(i32),
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientCost {
        item_id: i32,
        needed: i32,
        available: i32,
    },
    #[error("gacha pool {0} has no usable reward")]
    MissingReward(i32),
    #[error("gacha cumulative reward {0} is not selectable")]
    InvalidCumulativeReward(i32),
    #[error("no gacha cumulative reward is available")]
    CumulativeRewardUnavailable,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CheckinError {
    #[error("unknown check-in activity {0}")]
    UnknownActivity(i32),
    #[error("check-in activity {0} is not active")]
    Inactive(i32),
    #[error("unknown check-in day {day} for activity {aid}")]
    UnknownDay { aid: i32, day: i32 },
    #[error("check-in day {day} for activity {aid} is not available")]
    Unavailable { aid: i32, day: i32 },
    #[error("check-in day {day} for activity {aid} was already claimed")]
    AlreadyClaimed { aid: i32, day: i32 },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RoleMutationError {
    #[error("unknown role {0}")]
    UnknownRole(i32),
    #[error("skin {skin_id} is not unlocked for role {role_id}")]
    LockedSkin { role_id: i32, skin_id: i32 },
    #[error("invalid appearance skill key {0}")]
    InvalidAppearKey(i32),
    #[error("invalid role element {0}")]
    InvalidElement(i32),
    #[error("role {0} is not a main-character variant")]
    NotMainCharacter(i32),
    #[error("role {0} is already the current main character")]
    AlreadyMainCharacter(i32),
    #[error("current main character is missing")]
    MissingMainCharacter,
    #[error("unknown partner {0}")]
    UnknownPartner(i64),
    #[error("partner {0} is not equipped")]
    PartnerNotEquipped(i64),
    #[error("cannot swap a partner with itself")]
    SamePartner,
    #[error("partner lock value must be 0 or 1, got {0}")]
    InvalidPartnerLock(i32),
    #[error("role {0} is already at its level cap")]
    LevelCap(i32),
    #[error("invalid role EXP material {0}")]
    InvalidMaterial(i32),
    #[error("material {item_id} needs {needed}, only {available} available")]
    InsufficientMaterial {
        item_id: i32,
        needed: i32,
        available: i32,
    },
    #[error("role level-up needs {needed} gold, only {available} available")]
    InsufficientGold { needed: i32, available: i32 },
    #[error("role {0} must reach its current level cap before ranking up")]
    RankLevelRequired(i32),
    #[error("role {0} is already at maximum rank")]
    RankCap(i32),
    #[error("role rank-up needs world level {needed}, current level is {current}")]
    RankWorldLevelRequired { needed: i32, current: i32 },
    #[error("role {0} is already at maximum resonance")]
    ResonanceCap(i32),
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientItem {
        item_id: i32,
        needed: i32,
        available: i32,
    },
    #[error("unknown rank {level} for role {role_id}")]
    UnknownRank { role_id: i32, level: i32 },
    #[error("role {role_id} has not unlocked rank reward {level}")]
    RankRewardLocked { role_id: i32, level: i32 },
    #[error("role {role_id} rank reward {level} was already claimed")]
    RankRewardClaimed { role_id: i32, level: i32 },
    #[error("unknown talent position {position} for role {role_id}")]
    UnknownTalent { role_id: i32, position: i32 },
    #[error("talent position {position} for role {role_id} is already at maximum level")]
    TalentCap { role_id: i32, position: i32 },
    #[error("talent position {position} needs role rank {needed}, current rank is {current}")]
    TalentRankRequired {
        position: i32,
        needed: i32,
        current: i32,
    },
    #[error("talent position {position} needs role level {needed}, current level is {current}")]
    TalentLevelRequired {
        position: i32,
        needed: i32,
        current: i32,
    },
    #[error("talent material group {key} is invalid for role {role_id}")]
    InvalidTalentCost { role_id: i32, key: i32 },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PartnerProgressError {
    #[error("unknown partner {0}")]
    UnknownPartner(i64),
    #[error("partner {0} has no progression config")]
    MissingConfig(i32),
    #[error("partner {0} is already at its level cap")]
    LevelCap(i64),
    #[error("invalid partner EXP material {0}")]
    InvalidMaterial(i32),
    #[error("partner {0} must reach its current level cap before breaking through")]
    BreakLevelRequired(i64),
    #[error("partner {0} is already at maximum break rank")]
    BreakCap(i64),
    #[error("partner breakthrough needs world level {needed}, current level is {current}")]
    WorldLevelRequired { needed: i32, current: i32 },
    #[error("partner {0} is already at maximum skill level")]
    SkillCap(i64),
    #[error("partner {0} is already at maximum resonance")]
    ResonanceCap(i64),
    #[error("invalid resonance material partner {0}")]
    InvalidResonancePartner(i64),
    #[error("resonance material partner {0} is locked")]
    LockedResonancePartner(i64),
    #[error("resonance material partner {0} is equipped")]
    EquippedResonancePartner(i64),
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientItem {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum MallPurchaseError {
    #[error("unknown mall goods {0}")]
    Unknown(i32),
    #[error("mall goods {0} is not active")]
    Inactive(i32),
    #[error("mall purchase amount must be positive")]
    InvalidAmount,
    #[error("mall goods {id} allows at most {limit} per purchase")]
    SingleLimit { id: i32, limit: i32 },
    #[error("mall goods {id} purchase limit is {limit}")]
    BuyLimit { id: i32, limit: i32 },
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientCost {
        item_id: i32,
        needed: i32,
        available: i32,
    },
    #[error("mall goods {0} has no usable reward")]
    MissingReward(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RechargePurchaseError {
    #[error("unknown recharge product {0}")]
    Unknown(i32),
    #[error("recharge product {id} has unsupported type {kind}")]
    UnsupportedType { id: i32, kind: i32 },
    #[error("monthly card is already at its duration cap")]
    DurationCap,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ActivityLevelRewardError {
    #[error("unknown activity level reward {0}")]
    Unknown(i32),
    #[error("activity level reward {id} requires level {required}")]
    LevelLocked { id: i32, required: i32 },
    #[error("activity level reward {0} was already claimed")]
    AlreadyClaimed(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum LimitedLevelRewardError {
    #[error("the limited level activity is not available")]
    Inactive,
    #[error("unknown limited level reward {0}")]
    Unknown(i32),
    #[error("limited level reward {id} requires level {required}")]
    LevelLocked { id: i32, required: i32 },
    #[error("limited level reward {0} was already claimed")]
    AlreadyClaimed(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum BossRushError {
    #[error("boss rush {0} is not active")]
    Inactive(i32),
    #[error("unknown boss-rush port {0}")]
    UnknownPort(i32),
    #[error("boss-rush damage cannot be negative")]
    NegativeDamage,
    #[error("unknown role {0}")]
    UnknownRole(i32),
    #[error("unknown boss-rush reward {0}")]
    UnknownReward(i32),
    #[error("boss-rush reward {0} is not reached")]
    RewardNotReached(i32),
    #[error("boss-rush reward {0} was already claimed")]
    RewardAlreadyClaimed(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum BattlePassError {
    #[error("there is no active battle pass")]
    Inactive,
    #[error("unknown battle-pass task {0}")]
    UnknownTask(i32),
    #[error("battle-pass task {0} is incomplete")]
    IncompleteTask(i32),
    #[error("battle-pass task {0} was already claimed")]
    TaskAlreadyClaimed(i32),
    #[error("unknown battle-pass reward level {0}")]
    UnknownReward(i32),
    #[error("battle-pass reward level {0} is locked")]
    LockedReward(i32),
    #[error("battle-pass reward level {0} was already claimed")]
    RewardAlreadyClaimed(i32),
    #[error("battle-pass level increase must be positive")]
    InvalidLevelIncrease,
    #[error("battle-pass level cap is {0}")]
    LevelCap(i32),
    #[error("not enough item {item_id}: need {needed}, have {available}")]
    InsufficientCost {
        item_id: i32,
        needed: i32,
        available: i32,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum EquipmentError {
    #[error("unknown relic definition {0}")]
    UnknownRelic(i32),
    #[error("relic {0} is unfinished and unavailable in this client")]
    UnavailableRelic(i32),
    #[error("relic amount must be positive, got {0}")]
    InvalidRelicAmount(i32),
    #[error("relic {relic_id} cannot use main stat {word_id}")]
    InvalidRelicMainStat { relic_id: i32, word_id: i32 },
    #[error("relic {relic_id} needs {expected} initial substats, got {actual}")]
    InvalidRelicSubstatCount {
        relic_id: i32,
        expected: usize,
        actual: usize,
    },
    #[error("relic {relic_id} cannot use substat {word_id}")]
    InvalidRelicSubstat { relic_id: i32, word_id: i32 },
    #[error("relic substat effect {effect_id} exceeds its limit of {limit}")]
    RelicSubstatLimit { effect_id: i32, limit: i32 },
    #[error("relic {0} has no quality parameters")]
    MissingRelicParameters(i32),
    #[error("unknown equipment {0}")]
    UnknownEquipment(i64),
    #[error("unknown role {0}")]
    UnknownRole(i32),
    #[error("equipment {equip_id} belongs in slot {actual}, not {requested}")]
    WrongSlot {
        equip_id: i64,
        actual: i32,
        requested: i32,
    },
    #[error("equipment {first} and {second} use different slots")]
    SwapSlotMismatch { first: i64, second: i64 },
    #[error("equipment lock state must be 0 or 1")]
    InvalidLockState,
    #[error("equipment {0} is already at its level cap")]
    LevelCap(i64),
    #[error("invalid equipment EXP material {0}")]
    InvalidMaterial(i32),
    #[error("material {item_id} used the wrong instance id")]
    WrongItemInstance { item_id: i32 },
    #[error("material {item_id} needs {needed}, only {available} available")]
    InsufficientMaterial {
        item_id: i32,
        needed: i32,
        available: i32,
    },
    #[error("equipment level-up needs {needed} gold, only {available} available")]
    InsufficientGold { needed: i32, available: i32 },
    #[error("equipment {equip_id} has incomplete level configuration")]
    MissingLevelConfig { equip_id: i32 },
    #[error("equipment random-word slot {0} is locked or missing")]
    InvalidWordSlot(i32),
    #[error("equipment random-word material {0} is insufficient")]
    InsufficientWordCost(i32),
    #[error("equipment random-word candidate {0} is invalid")]
    InvalidWord(i32),
    #[error("equipment has no pending random word for slot {0}")]
    NoPendingWord(i32),
    #[error("unknown equipment group {0}")]
    UnknownGroup(i64),
    #[error("equipment group id cannot be negative")]
    InvalidGroupId,
    #[error("equipment group contains more than one item for slot {0}")]
    DuplicateGroupSlot(i32),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RiftError {
    #[error("rift {0} is not active")]
    Inactive(i32),
    #[error("no rift run is active")]
    NoActiveRun,
    #[error("rift buff {0} does not belong to this event")]
    UnknownBuff(i32),
    #[error("rift buff {buff_id} has no choice {choice}")]
    UnknownBuffChoice { buff_id: i32, choice: i32 },
    #[error("rift buff {buff_id} needs buff score {needed}")]
    LockedBuff { buff_id: i32, needed: i32 },
    #[error("unknown role {0}")]
    UnknownRole(i32),
    #[error("not enough Rift tickets: need {needed}, have {available}")]
    InsufficientTickets { needed: i32, available: i32 },
    #[error("rift stage {0} is not part of this event")]
    UnknownStage(i32),
    #[error("rift stage {0} is already the final stage")]
    FinalStage(i32),
    #[error("rift stage {0} is not the final stage")]
    NotFinalStage(i32),
    #[error("rift duration cannot be negative")]
    NegativeDuration,
    #[error("unknown rift task {0}")]
    UnknownTask(i32),
    #[error("rift task {0} is not complete")]
    TaskNotComplete(i32),
    #[error("rift task {0} was already claimed")]
    TaskAlreadyClaimed(i32),
}
