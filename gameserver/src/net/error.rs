#[derive(Debug, thiserror::Error)]
pub(crate) enum NetworkError {
    #[error(transparent)]
    Packet(#[from] common::network::error::PacketError),
    #[error(transparent)]
    Crypto(#[from] common::network::crypto::CryptoError),
    #[error(transparent)]
    ProtobufDecode(#[from] protocol::prost::DecodeError),
    #[error(transparent)]
    ProtobufEncode(#[from] protocol::prost::EncodeError),
    #[error(transparent)]
    Database(#[from] database::Error),
    #[error("encrypted packet received before key exchange")]
    MissingKeyExchange,
    #[error("duplicate key exchange")]
    DuplicateKeyExchange,
    #[error("unknown city guide {0}")]
    UnknownCityGuide(String),
    #[error("unknown interaction object {0}")]
    UnknownInteract(String),
    #[error("invalid interaction: {0}")]
    InvalidInteraction(String),
    #[error("invalid city challenge: {0}")]
    InvalidCityChallenge(String),
    #[error("album {0} is not unlocked")]
    LockedAlbum(i32),
    #[error("invalid formation: {0}")]
    InvalidFormation(String),
    #[error("invalid naming request: {0}")]
    InvalidNaming(String),
    #[error("invalid heat exchange: {0}")]
    InvalidHeatExchange(String),
    #[error("invalid currency exchange: {0}")]
    InvalidExchange(String),
    #[error("invalid mission claim: {0}")]
    InvalidMission(String),
    #[error("invalid task completion: {0}")]
    InvalidTask(String),
    #[error("invalid trial action: {0}")]
    InvalidTrial(String),
    #[error("invalid port progression: {0}")]
    InvalidPort(String),
    #[error("invalid dungeon action: {0}")]
    InvalidDungeon(String),
    #[error("invalid daily task action: {0}")]
    InvalidDailyTask(String),
    #[error("invalid gold coin action: {0}")]
    InvalidGoldCoin(String),
    #[error("invalid collection resource action: {0}")]
    InvalidCollectionResource(String),
    #[error("invalid collection action: {0}")]
    InvalidCollection(String),
    #[error("invalid monster point action: {0}")]
    InvalidMonsterPoint(String),
    #[error("unknown region {0}")]
    UnknownRegion(i32),
    #[error("invalid region action: {0}")]
    InvalidRegion(String),
    #[error("invalid reward box action: {0}")]
    InvalidRewardBox(String),
    #[error("invalid achievement action: {0}")]
    InvalidAchievement(String),
    #[error("invalid activity action: {0}")]
    InvalidActivity(String),
    #[error("invalid gacha action: {0}")]
    InvalidGacha(String),
    #[error("invalid check-in action: {0}")]
    InvalidCheckin(String),
    #[error("invalid role mutation: {0}")]
    InvalidRoleMutation(String),
    #[error("invalid saved role attributes: {0}")]
    InvalidRoleAttr(String),
    #[error("invalid inventory action: {0}")]
    InvalidInventory(String),
    #[error("invalid partner progression: {0}")]
    InvalidPartnerProgression(String),
    #[error("invalid battle-pass action: {0}")]
    InvalidBattlePass(String),
    #[error("invalid mall purchase: {0}")]
    InvalidMallPurchase(String),
    #[error("invalid ship tags: {0}")]
    InvalidShipTags(String),
    #[error("invalid disassembly: {0}")]
    InvalidDisassembly(String),
    #[error("invalid boss-rush action: {0}")]
    InvalidBossRush(String),
    #[error("invalid activity-boss action: {0}")]
    InvalidActivityBoss(String),
    #[error("invalid activity-challenge action: {0}")]
    InvalidActivityChallenge(String),
    #[error("invalid version-challenge action: {0}")]
    InvalidVersionChallenge(String),
    #[error("invalid more-team-challenge action: {0}")]
    InvalidMoreTeamChallenge(String),
    #[error("invalid team-core action: {0}")]
    InvalidTeamCore(String),
    #[error("invalid team-equipment action: {0}")]
    InvalidTeamEquip(String),
    #[error("invalid skill-stone action: {0}")]
    InvalidSkillStone(String),
    #[error("invalid feature action: {0}")]
    InvalidFeature(String),
    #[error("invalid equipment action: {0}")]
    InvalidEquipment(String),
    #[error("invalid rift action: {0}")]
    InvalidRift(String),
    #[error("invalid Rogue action: {0}")]
    InvalidRouge(String),
    #[error("invalid favor action: {0}")]
    InvalidFavor(String),
    #[error("invalid SMS action: {0}")]
    InvalidSms(String),
    #[error("invalid social action: {0}")]
    InvalidSocial(String),
    #[error("profile {profile_type} does not contain id {id}")]
    LockedProfile { profile_type: i32, id: i32 },
    #[error("game command received before authentication")]
    Unauthenticated,
    #[error("command handler completed without producing a reply")]
    MissingHandlerReply,
}

pub(crate) type NetworkResult<T> = Result<T, NetworkError>;
